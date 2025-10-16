use roxmltree::Document;

pub struct File4 {
    id: String,
    reporter: Reporter,
    issuer: Issuer,
    non_derivative: Vec<NonDerivative>,
    derivative: Vec<Derivative>,
}

pub struct Reporter {
    name: String,
    cik: String,
    relation: Relation,
}
pub struct Issuer {
    name: String,
    cik: String,
    symbol: String,
}

pub enum Relation {
    Director,
    Officer(String),
    Owner,
    Other(String),
}

pub struct NonDerivative {
    title: String,
    date: Option<String>,
    tx_codes: Option<Vec<TransactionCode>>,
    tx_data: Option<TransactionData>,
    owned: i32,
    ownership: Ownership,
}

pub struct Derivative {
    title: String,
    date: Option<String>,
    tx_codes: Option<Vec<TransactionCode>>,
    count: Option<DerivativeNumber>,
    underlying: Option<Underlying>,
    price: Option<f32>,
    owned: i32,
    ownership: Ownership,
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
    amount: i32,
    acqired: bool,
    price: f32,
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
    title: String,
    price: f32,
}

impl File4 {
    pub fn new(id: String, data: String) -> anyhow::Result<Self> {
        Document::parse(&data)?;
        Ok(Self {
            id,
            reporter: todo!(),
            issuer: todo!(),
            non_derivative: todo!(),
            derivative: todo!(),
        })
    }
}
