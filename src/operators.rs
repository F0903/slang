use crate::identifiable::Identifiable;

pub enum Operation {
    Plus(Operator),
    Multiply(Operator),
    Divide(Operator),
}

impl Identifiable for Operation {
    fn get_identifier(&self) -> &'static str {
        match self {
            Operation::Plus(x) => x.get_identifier(),
            Operation::Multiply(x) => x.get_identifier(),
            Operation::Divide(x) => x.get_identifier(),
        }
    }
}

pub struct Operator {
    identifier: &'static str,
}

impl Identifiable for Operator {
    fn get_identifier(&self) -> &'static str {
        self.identifier
    }
}

pub const OPERATORS: &[Operation] = &[
    Operation::Plus(Operator { identifier: "+" }),
    Operation::Multiply(Operator { identifier: "*" }),
    Operation::Divide(Operator { identifier: "/" }),
];
