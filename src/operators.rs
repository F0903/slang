use crate::types::Identifiable;

pub trait ContainsOperator {
    fn get_op(&self) -> &Operator;
}

pub trait OpPriority: ContainsOperator {
    fn get_op_priority(&self) -> OperatorPriority;
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPriority {
    Low,
    Medium,
    High,
    Highest,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Plus(Operator),
    Minus(Operator),
    Multiply(Operator),
    Divide(Operator),
    Equal(Operator),
    LessThan(Operator),
    LessOrEq(Operator),
    MoreThan(Operator),
    MoreOrEq(Operator),
    NoOp(Operator),
}

impl ContainsOperator for Operation {
    fn get_op(&self) -> &Operator {
        match self {
            Operation::Plus(x)
            | Operation::Minus(x)
            | Operation::Multiply(x)
            | Operation::Divide(x)
            | Operation::Equal(x)
            | Operation::LessThan(x)
            | Operation::LessOrEq(x)
            | Operation::MoreThan(x)
            | Operation::MoreOrEq(x)
            | Operation::NoOp(x) => x,
        }
    }
}

impl OpPriority for Operation {
    fn get_op_priority(&self) -> OperatorPriority {
        self.get_op().priority
    }
}

impl Identifiable for Operation {
    fn get_identifier(&self) -> &'static str {
        self.get_op().get_identifier()
    }
}

#[derive(Debug, Clone)]
pub struct Operator {
    identifier: &'static str,
    priority: OperatorPriority,
}

impl Identifiable for Operator {
    fn get_identifier(&self) -> &'static str {
        self.identifier
    }
}

pub const PLUS: Operation = Operation::Plus(Operator {
    identifier: "+",
    priority: OperatorPriority::Low,
});

pub const MINUS: Operation = Operation::Minus(Operator {
    identifier: "-",
    priority: OperatorPriority::Low,
});

pub const MULTIPLY: Operation = Operation::Multiply(Operator {
    identifier: "*",
    priority: OperatorPriority::Medium,
});

pub const DIVIDE: Operation = Operation::Divide(Operator {
    identifier: "/",
    priority: OperatorPriority::Medium,
});

pub const EQUAL: Operation = Operation::Equal(Operator {
    identifier: "==",
    priority: OperatorPriority::Low,
});

pub const LESSTHAN: Operation = Operation::LessThan(Operator {
    identifier: "<",
    priority: OperatorPriority::Low,
});

pub const LESSOREQ: Operation = Operation::LessOrEq(Operator {
    identifier: "<=",
    priority: OperatorPriority::Low,
});

pub const MORETHAN: Operation = Operation::MoreThan(Operator {
    identifier: ">",
    priority: OperatorPriority::Low,
});

pub const MOREOREQ: Operation = Operation::MoreOrEq(Operator {
    identifier: ">=",
    priority: OperatorPriority::Low,
});

pub const NOOP: Operation = Operation::NoOp(Operator {
    identifier: "noop",
    priority: OperatorPriority::Low,
});

// REMEMBER TO DEFINE OPS HERE AS WELL.
pub const OPERATORS: &[Operation] = &[
    PLUS, MINUS, MULTIPLY, DIVIDE, EQUAL, LESSTHAN, LESSOREQ, MORETHAN, MOREOREQ, NOOP,
];
