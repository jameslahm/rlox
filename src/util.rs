pub fn is_digit(c:u8) -> bool {
    c>=b'0' && c<=b'9'
}

pub fn is_alpha(c:u8) -> bool {
    (c>= b'a' && c<= b'z') || (c>=b'A' && c<=b'Z') || (c==b'_')
}

#[macro_export]
macro_rules! matches {
    ($e:expr,$p:pat) => {
        match $e {
            $p => true,
            _ => false
        }
    };
}

#[macro_export]
macro_rules! binary_op {
    ($self:ident,$val_type:ident,$op:tt) => {
        if let Value::Double(right_v) = $self.peek(0) {
            if let Value::Double(left_v) = $self.peek(1) {
                $self.stack.push(Value::$val_type(left_v $op right_v));
                // Pop values
                $self.get_stack_value()?;
                $self.get_stack_value()?;

                continue;
            }
        }
        return Err(VmError::RuntimeError(error::OPERAND_MUST_BE_NUMBER));
    };
}