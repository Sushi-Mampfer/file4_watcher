use std::str::FromStr;

use anyhow::anyhow;
use regex::Regex;
use roxmltree::{Children, Document};

pub struct File4 {
    pub id: String,
    pub file_name: String,
    pub reporters: Vec<Reporter>,
    pub issuer: Issuer,
    pub non_derivative: Vec<NonDerivative>,
    pub derivative: Vec<Derivative>,
}

pub struct Reporter {
    pub name: String,
    pub cik: String,
    pub relation: Relations,
}
pub struct Issuer {
    pub name: String,
    pub cik: String,
    pub symbol: String,
}

pub struct Relations {
    pub relations: Vec<Relation>,
    pub title: Option<String>,
}
pub enum Relation {
    Director,
    Officer,
    Owner,
    Other,
}

pub struct NonDerivative {
    pub title: String,
    pub date: Option<String>,
    pub tx_codes: Option<Vec<TransactionCode>>,
    pub tx_data: Option<TransactionData>,
    pub owned: f32,
    pub ownership: Ownership,
}

pub struct Derivative {
    pub title: String,
    pub date: Option<String>,
    pub tx_codes: Option<Vec<TransactionCode>>,
    pub count: Option<DerivativeNumber>,
    pub underlying: Option<Underlying>,
    pub price: Option<f32>,
    pub owned: f32,
    pub ownership: Ownership,
}

pub enum TransactionCode {
    // General Transaction Codes
    P, // Open market or private purchase of non-derivative or derivative security
    S, // Open market or private sale of non-derivative or derivative security
    V, // Transaction voluntarily reported earlier than required
    // Rule 16b-3 Transaction Codes
    A, // Grant, award or other acquisition pursuant to Rule 16b-3(d)
    D, // Disposition to the issuer of issuer equity securities pursuant to Rule 16b-3(e)
    F, // Payment of exercise price or tax liability by delivering or withholding securities incident to the receipt, exercise or vesting of a security issued in accordance with Rule 16b-3
    I, // Discretionary transaction in accordance with Rule 16b-3(f) resulting in acquisition or disposition of issuer securities
    M, // Exercise or conversion of derivative security exempted pursuant to Rule 16b-3
    // Derivative Securities Codes (Except for transactions exempted pursuant to Rule 16b-3)
    C, // Conversion of derivative security
    E, // Expiration of short derivative position
    H, // Expiration (or cancellation) of long derivative position with value received
    O, // Exercise of out-of-the-money derivative security
    X, // Exercise of in-the-money or at-the-money derivative security
    // Other Section 16(b) Exempt Transaction and Small Acquisition Codes (except for Rule 16b-3 codes above)
    G, // Bona fide gift
    L, // Small acquisition under Rule 16a-6
    W, // Acquisition or disposition by will or the laws of descent and distribution
    Z, // Deposit into or withdrawal from voting trust
    // Other Transaction Codes
    J, // Other acquisition or disposition (describe transaction)
    K, // Transaction in equity swap or instrument with similar characteristics
    U, // Disposition pursuant to a tender of shares in a change of control transaction
}

pub struct TransactionData {
    pub amount: i32,
    pub acqired: bool,
    pub price: f32,
}

pub enum Ownership {
    Direct,
    Indirect(String),
}

pub enum DerivativeNumber {
    Acquired(i32),
    Disposed(i32),
}

pub struct Underlying {
    pub title: String,
    pub price: f32,
}

impl File4 {
    pub fn new(data: String) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"(?s)ACCESSION NUMBER:\s+([a-zA-Z0-9-]*).*?<FILENAME>([a-zA-Z0-9-_]*\.xml).*?<XML>\n(.*?)\n<\/XML>",
        )?;
        let caps = re.captures(&data).ok_or(anyhow!("No caps found"))?;
        let id = caps
            .get(1)
            .ok_or(anyhow!("No accession number found"))?
            .as_str()
            .to_string();

        let file_name = caps
            .get(2)
            .ok_or(anyhow!("No file name found"))?
            .as_str()
            .to_string();

        let xml = caps.get(3).ok_or(anyhow!("No xml found"))?.as_str();

        let doc = Document::parse(xml)?;

        let mut reporters = Vec::new();

        for i in doc
            .descendants()
            .filter(|n| n.has_tag_name("reportingOwner"))
        {
            let mut reporter_id = i
                .children()
                .find(|n| n.has_tag_name("reportingOwnerId"))
                .ok_or(anyhow!("No reporter id found"))?
                .children();

            let mut relationship = i
                .children()
                .find(|n| n.has_tag_name("reportingOwnerRelationship"))
                .ok_or(anyhow!("No relation found"))?
                .children();

            let mut relations = Vec::new();

            match relationship.find(|n| n.has_tag_name("isDirector")) {
                Some(n) => match n.text() {
                    Some(t) => {
                        if t == "true" {
                            relations.push(Relation::Director);
                        } else {
                            ()
                        }
                    }
                    None => (),
                },
                None => (),
            }

            match relationship.find(|n| n.has_tag_name("isOfficer")) {
                Some(n) => match n.text() {
                    Some(t) => {
                        if t == "true" {
                            relations.push(Relation::Officer);
                        } else {
                            ()
                        }
                    }
                    None => (),
                },
                None => (),
            };

            match relationship.find(|n| n.has_tag_name("isTenPercentOwner")) {
                Some(n) => match n.text() {
                    Some(t) => {
                        if t == "true" {
                            relations.push(Relation::Owner);
                        } else {
                            ()
                        }
                    }
                    None => (),
                },
                None => (),
            };

            match relationship.find(|n| n.has_tag_name("isOther")) {
                Some(n) => match n.text() {
                    Some(t) => {
                        if t == "true" {
                            relations.push(Relation::Other);
                        } else {
                            ()
                        }
                    }
                    None => (),
                },
                None => (),
            };

            let title = relationship
                .find(|n| n.has_tag_name("officerTitle"))
                .map(|n| n.text())
                .unwrap_or_default()
                .map(|t| t.to_owned());

            let relation = Relations { relations, title };

            reporters.push(Reporter {
                cik: reporter_id
                    .find(|n| n.has_tag_name("rptOwnerCik"))
                    .ok_or(anyhow!("No reporter cik found"))?
                    .text()
                    .ok_or(anyhow!("No reporter cik text found"))?
                    .to_owned(),
                name: reporter_id
                    .clone()
                    .find(|n| n.has_tag_name("rptOwnerName"))
                    .ok_or(anyhow!("No reporter name found"))?
                    .text()
                    .ok_or(anyhow!("No reporter name text found"))?
                    .to_owned(),
                relation,
            });
        }

        let mut issuer = doc
            .descendants()
            .find(|n| n.has_tag_name("issuer"))
            .ok_or(anyhow!("No issuer found"))?
            .children();

        let issuer = Issuer {
            cik: issuer
                .find(|n| n.has_tag_name("issuerCik"))
                .ok_or(anyhow!("No issuer cik found"))?
                .text()
                .ok_or(anyhow!("No issuer cik text found"))?
                .to_owned(),
            name: issuer
                .find(|n| n.has_tag_name("issuerName"))
                .ok_or(anyhow!("No issuer name found"))?
                .text()
                .ok_or(anyhow!("No issuer name text found"))?
                .to_owned(),
            symbol: issuer
                .find(|n| n.has_tag_name("issuerTradingSymbol"))
                .ok_or(anyhow!("No issuer symbol found"))?
                .text()
                .ok_or(anyhow!("No issuer symbol text found"))?
                .to_owned(),
        };

        let mut non_derivative: Vec<NonDerivative> = Vec::new();

        if let Some(non_derivatives) = doc
            .descendants()
            .find(|n| n.has_tag_name("nonDerivativeTable"))
        {
            for i in non_derivatives.children() {
                if i.tag_name().name() == "" {
                    continue;
                }

                let title = i
                    .children()
                    .find(|n| n.has_tag_name("securityTitle"))
                    .ok_or(anyhow!("No non derivative title found"))?
                    .first_child()
                    .ok_or(anyhow!("No non derivative title value found"))?
                    .text()
                    .ok_or(anyhow!("No non derivative title value text found"))?
                    .to_owned();

                let date = i
                    .children()
                    .find(|n| n.has_tag_name("transactionDate"))
                    .map(|n| n.first_child())
                    .unwrap_or_default()
                    .map(|n| n.text())
                    .unwrap_or_default()
                    .map(|t| t.to_owned());

                let tx_codes = i
                    .children()
                    .find(|n| n.has_tag_name("transactionCoding"))
                    .map(|n| n.children().find(|n| n.has_tag_name("transactionCode")))
                    .unwrap_or_default()
                    .map(|n| n.text())
                    .unwrap_or_default()
                    .map(|t| TransactionCode::from_str(t))
                    .unwrap_or_default();

                let tx_data = i
                    .children()
                    .find(|n| n.has_tag_name("transactionAmounts"))
                    .map(|n| n.children())
                    .map(|t| TransactionData::from_children(t))
                    .unwrap_or_default();

                let owned = i
                    .children()
                    .find(|n| n.has_tag_name("postTransactionAmounts"))
                    .ok_or(anyhow!("No non derivative owned amount found"))?
                    .children()
                    .find(|n| n.has_tag_name("sharesOwnedFollowingTransaction"))
                    .ok_or(anyhow!("No non derivative owned amount 2 found"))?
                    .children()
                    .find(|n| n.has_tag_name("value"))
                    .ok_or(anyhow!("No non derivative owned amount value found"))?
                    .text()
                    .ok_or(anyhow!("No non derivative owned amount value text found"))?
                    .parse()?;

                let mut ownership = i
                    .children()
                    .find(|n| n.has_tag_name("ownershipNature"))
                    .ok_or(anyhow!("No non derivative ownership found"))?
                    .children();

                let ownership = match ownership
                    .find(|n| n.has_tag_name("directOrIndirectOwnership"))
                    .ok_or(anyhow!("No non derivative ownership bool found"))?
                    .children()
                    .find(|n| n.has_tag_name("value"))
                    .ok_or(anyhow!("No non derivative ownership value found"))?
                    .text()
                    .ok_or(anyhow!("No non derivative ownership value text found"))?
                {
                    "D" => Ownership::Direct,
                    _ => Ownership::Indirect(
                        ownership
                            .find(|n| n.has_tag_name("natureOfOwnership"))
                            .ok_or(anyhow!("No non derivative ownership nature found"))?
                            .first_child()
                            .ok_or(anyhow!("No non derivative ownership nature value found"))?
                            .text()
                            .ok_or(anyhow!(
                                "No non derivative ownership nature value text found"
                            ))?
                            .to_owned(),
                    ),
                };

                non_derivative.push(NonDerivative {
                    title,
                    date,
                    tx_codes,
                    tx_data,
                    owned,
                    ownership,
                });
            }
        };

        let mut derivative: Vec<Derivative> = Vec::new();

        if let Some(derivatives) = doc
            .descendants()
            .find(|n| n.has_tag_name("nonDerivativeTable"))
        {
            for i in derivatives.children() {
                if i.tag_name().name() == "" {
                    continue;
                }

                let title = i
                    .children()
                    .find(|n| n.has_tag_name("securityTitle"))
                    .ok_or(anyhow!("No derivative title found"))?
                    .first_child()
                    .ok_or(anyhow!("No derivative title value found"))?
                    .text()
                    .ok_or(anyhow!("No derivative title value text found"))?
                    .to_owned();

                let date = i
                    .children()
                    .find(|n| n.has_tag_name("transactionDate"))
                    .map(|n| n.first_child())
                    .unwrap_or_default()
                    .map(|n| n.text())
                    .unwrap_or_default()
                    .map(|t| t.to_owned());

                let tx_codes = i
                    .children()
                    .find(|n| n.has_tag_name("transactionCoding"))
                    .map(|n| n.children().find(|n| n.has_tag_name("transactionCode")))
                    .unwrap_or_default()
                    .map(|n| n.text())
                    .unwrap_or_default()
                    .map(|t| TransactionCode::from_str(t))
                    .unwrap_or_default();

                let count = i
                    .children()
                    .find(|n| n.has_tag_name("transactionAmounts"))
                    .map(|n| DerivativeNumber::from_children(n.children()))
                    .unwrap_or_default();

                let underlying = i
                    .children()
                    .find(|n| n.has_tag_name("underlyingSecurity"))
                    .map(|n| Underlying::from_children(n.children()))
                    .unwrap_or_default();

                let price = i
                    .children()
                    .find(|n| n.has_tag_name("transactionAmounts"))
                    .map(|n| {
                        n.children()
                            .find(|n| n.has_tag_name("transactionPricePerShare"))
                    })
                    .unwrap_or_default()
                    .map(|n| n.first_child())
                    .unwrap_or_default()
                    .map(|n| n.text())
                    .unwrap_or_default()
                    .map(|t| t.parse().ok())
                    .unwrap_or_default();

                let owned = i
                    .children()
                    .find(|n| n.has_tag_name("postTransactionAmounts"))
                    .ok_or(anyhow!("No derivative owned amount found"))?
                    .children()
                    .find(|n| n.has_tag_name("sharesOwnedFollowingTransaction"))
                    .ok_or(anyhow!("No derivative owned amount 2 found"))?
                    .children()
                    .find(|n| n.has_tag_name("value"))
                    .ok_or(anyhow!("No derivative owned amount value found"))?
                    .text()
                    .ok_or(anyhow!("No derivative owned amount value text found"))?
                    .parse()?;

                let mut ownership = i
                    .children()
                    .find(|n| n.has_tag_name("ownershipNature"))
                    .ok_or(anyhow!("No non derivative ownership found"))?
                    .children();

                let ownership = match ownership
                    .find(|n| n.has_tag_name("directOrIndirectOwnership"))
                    .ok_or(anyhow!("No non derivative ownership bool found"))?
                    .children()
                    .find(|n| n.has_tag_name("value"))
                    .ok_or(anyhow!("No non derivative ownership value found"))?
                    .text()
                    .ok_or(anyhow!("No non derivative ownership value text found"))?
                {
                    "D" => Ownership::Direct,
                    _ => Ownership::Indirect(
                        ownership
                            .find(|n| n.has_tag_name("natureOfOwnership"))
                            .ok_or(anyhow!("No non derivative ownership nature found"))?
                            .first_child()
                            .ok_or(anyhow!("No non derivative ownership nature value found"))?
                            .text()
                            .ok_or(anyhow!(
                                "No non derivative ownership nature value text found"
                            ))?
                            .to_owned(),
                    ),
                };

                derivative.push(Derivative {
                    title,
                    date,
                    tx_codes,
                    count,
                    underlying,
                    price,
                    owned,
                    ownership,
                });
            }
        };

        Ok(Self {
            id,
            file_name,
            reporters,
            issuer,
            non_derivative,
            derivative,
        })
    }
}

impl TransactionCode {
    pub fn from_str(codes: &str) -> Option<Vec<Self>> {
        let mut out = Vec::new();
        for c in codes.chars() {
            out.push(match c {
                'P' => Self::P,
                'S' => Self::S,
                'V' => Self::V,
                'A' => Self::A,
                'D' => Self::D,
                'F' => Self::F,
                'I' => Self::I,
                'M' => Self::M,
                'C' => Self::C,
                'E' => Self::E,
                'H' => Self::H,
                'O' => Self::O,
                'X' => Self::X,
                'G' => Self::G,
                'L' => Self::L,
                'W' => Self::W,
                'Z' => Self::Z,
                'J' => Self::J,
                'K' => Self::K,
                'U' => Self::U,
                _ => return None,
            });
        }
        Some(out)
    }
}

impl TransactionData {
    pub fn from_children(mut children: Children) -> Option<Self> {
        let Some(amount) = children
            .find(|n| n.has_tag_name("transactionAmounts"))
            .map(|n| n.first_child())
            .unwrap_or_default()
            .map(|n| n.text())
            .unwrap_or_default()
            .map(|t| t.parse().ok())
            .unwrap_or_default()
        else {
            return None;
        };

        let Some(acqired) = children
            .find(|n| n.has_tag_name("transactionAcquiredDisposedCode"))
            .map(|n| n.first_child())
            .unwrap_or_default()
            .map(|n| n.text())
            .unwrap_or_default()
            .map(|t| t == "D")
        else {
            return None;
        };

        let Some(price) = children
            .find(|n| n.has_tag_name("transactionPricePerShare"))
            .map(|n| n.first_child())
            .unwrap_or_default()
            .map(|n| n.text())
            .unwrap_or_default()
            .map(|t| t.parse().ok())
            .unwrap_or_default()
        else {
            return None;
        };

        Some(Self {
            amount,
            acqired,
            price,
        })
    }
}

impl DerivativeNumber {
    pub fn from_children(mut children: Children) -> Option<Self> {
        let count = children
            .find(|n| n.has_tag_name("transactionShares"))?
            .first_child()?
            .text()?
            .parse()
            .ok()?;
        Some(
            match children
                .find(|n| n.has_tag_name("transactionAcquiredDisposedCode"))?
                .first_child()?
                .text()?
            {
                "A" => Self::Acquired(count),
                _ => Self::Disposed(count),
            },
        )
    }
}

impl Underlying {
    pub fn from_children(mut children: Children) -> Option<Self> {
        Some(Self {
            title: children
                .find(|n| n.has_tag_name("underlyingSecurityTitle"))?
                .first_child()?
                .text()?
                .to_owned(),
            price: children
                .find(|n| n.has_tag_name("underlyingSecurityShares"))?
                .first_child()?
                .text()?
                .parse()
                .ok()?,
        })
    }
}
