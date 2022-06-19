use crate::operators::{self, OpPriority, Operation};
use crate::types::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub(super) struct SubExpression {
    pub(super) value: Value,
    pub(super) op: Operation,
    pub(super) next: Option<Box<Self>>,
}

impl SubExpression {
    fn remove_next_from_chain(&mut self) {
        let next: SubExpression;
        {
            let next_temp = match &self.next {
                None => return,
                Some(x) => x,
            };
            next = (**next_temp).clone();
        }

        let new_next = match next.next {
            None => {
                self.next = None;
                return;
            }
            Some(x) => x,
        };
        self.next = Some(new_next);
    }

    pub fn evaluate(&mut self) -> Result<Value> {
        let next = match &mut self.next {
            None => return Ok(self.value.clone()),
            Some(x) => x,
        };

        let my_priority = self.op.get_op_priority();
        let next_priority = next.op.get_op_priority();

        let mut set_plus_op = false;
        let next_value;
        if my_priority < next_priority {
            next_value = next.evaluate()?;
        } else {
            next_value = next.value.clone();
            if matches!(next.op, Operation::Minus(_)) && matches!(self.op, Operation::Plus(_)) {
                self.op = operators::MINUS;
            }
            set_plus_op = true;
        }

        self.value = self.value.perform_op(&self.op, &next_value)?;
        self.remove_next_from_chain();

        if set_plus_op {
            self.op = operators::PLUS
        }

        self.evaluate()
    }
}
