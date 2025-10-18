use anyhow::anyhow;
use regex::Regex;
use roxmltree::Document;

pub struct File4 {
    pub id: String,
    pub file_name: String,
    pub reporter: Reporter,
    pub issuer: Issuer,
    pub non_derivative: Vec<NonDerivative>,
    pub derivative: Vec<Derivative>,
}

pub struct Reporter {
    pub name: String,
    pub cik: String,
    pub relation: Relation,
}
pub struct Issuer {
    pub name: String,
    pub cik: String,
    pub symbol: String,
}

pub enum Relation {
    Director,
    Officer(String),
    Owner,
    Other(String),
}

pub struct NonDerivative {
    pub title: String,
    pub date: Option<String>,
    pub tx_codes: Option<Vec<TransactionCode>>,
    pub tx_data: Option<TransactionData>,
    pub owned: i32,
    pub ownership: Ownership,
}

pub struct Derivative {
    pub title: String,
    pub date: Option<String>,
    pub tx_codes: Option<Vec<TransactionCode>>,
    pub count: Option<DerivativeNumber>,
    pub underlying: Option<Underlying>,
    pub price: Option<f32>,
    pub owned: i32,
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
            r"(?s)ACCESSION NUMBER:\s+([a-zA-Z0-9-]*).*?<FILENAME>([a-zA-Z0-9-]*\.xml).*?<XML>\n(.*?)\n<\/XML>",
        )?;
        let caps = re.captures(&data).ok_or(anyhow!("No caps found"))?;
        let id = caps
            .get(0)
            .ok_or(anyhow!("No accession number found"))?
            .as_str()
            .to_string();

        let file_name = caps
            .get(1)
            .ok_or(anyhow!("No file name found"))?
            .as_str()
            .to_string();

        let xml = caps.get(2).ok_or(anyhow!("No xml found"))?.as_str();

        let doc = Document::parse(xml)?;

        let mut reporter = doc
            .descendants()
            .find(|n| n.has_tag_name("reportingOwner"))
            .ok_or(anyhow!("No reporter found"))?
            .children();

        let mut reporter_id = reporter
            .find(|n| n.has_tag_name("reportingOwnerId"))
            .ok_or(anyhow!("No reporter id"))?
            .children();

        let mut issuer = doc
            .descendants()
            .find(|n| n.has_tag_name("issuer"))
            .ok_or(anyhow!("No issuer found"))?
            .children();

        let issuer = Issuer {
            name: issuer
                .find(|n| n.has_tag_name("issuerName"))
                .ok_or(anyhow!("No issuer name found"))?
                .text()
                .ok_or(anyhow!("No issuer name text found"))?
                .to_owned(),
            cik: issuer
                .find(|n| n.has_tag_name("issuerCik"))
                .ok_or(anyhow!("No issuer cik found"))?
                .text()
                .ok_or(anyhow!("No issuer cik text found"))?
                .to_owned(),
            symbol: issuer
                .find(|n| n.has_tag_name("issuerTradingSymbol"))
                .ok_or(anyhow!("No issuer symbol found"))?
                .text()
                .ok_or(anyhow!("No issuer symbol text found"))?
                .to_owned(),
        };

        Ok(Self {
            id,
            file_name,
            reporter: todo!(),
            issuer,
            non_derivative: todo!(),
            derivative: todo!(),
        })
    }
}
