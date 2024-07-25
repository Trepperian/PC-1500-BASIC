use std::{collections::HashMap, ptr};

use crate::ast::{self, ExpressionVisitor, ProgramVisitor, StatementVisitor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // Logical
    And,
    Or,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Arithmetic
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            // Logical
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
            // Comparison
            BinaryOperator::Eq => write!(f, "=="),
            BinaryOperator::Ne => write!(f, "!="),
            BinaryOperator::Lt => write!(f, "<"),
            BinaryOperator::Le => write!(f, "<="),
            BinaryOperator::Gt => write!(f, ">"),
            BinaryOperator::Ge => write!(f, ">="),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operand {
    Variable { id: u32 },
    IndirectVariable { id: u32 },

    NumberLiteral { value: i32 },
    IndirectNumberLiteral { value: i32 },
}

impl Operand {
    pub fn variable_id(&self) -> Option<u32> {
        match self {
            Operand::Variable { id } => Some(*id),
            _ => None,
        }
    }

    pub fn number_literal_value(&self) -> Option<i32> {
        match self {
            Operand::NumberLiteral { value } => Some(*value),
            _ => None,
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Variable { id } => write!(f, "v{}", id),
            Operand::NumberLiteral { value } => write!(f, "{}", value),
            Operand::IndirectVariable { id } => write!(f, "*v{}", id),
            Operand::IndirectNumberLiteral { value } => write!(f, "*{}", value),
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Expression {
    left: Operand,
    op: BinaryOperator,
    right: Operand,
    dest: Operand,
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} = {} {} {};",
            self.dest, self.left, self.op, self.right
        )
    }
}

pub enum Hir { 
    Expression(Expression),
    Copy { src: Operand, dest: Operand },
    // Control flow
    Goto { label: u32 },
    GoSub { label: u32 },
    Label { id: u32 },
    Return,
    If { condition: Operand, label: u32 },
    // intrinsics
    Print { operand: Operand },
    Input { dest: Operand },
}

impl std::fmt::Display for Hir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hir::Copy { src, dest } => write!(f, "{} = {};", dest, src),
            Hir::Expression(expr) => write!(f, "{}", expr),
            Hir::Goto { label } => write!(f, "goto l{}", label),
            Hir::GoSub { label } => write!(f, "gosub l{}", label),
            Hir::Label { id } => write!(f, "l{}:", id),
            Hir::Return => write!(f, "return"),
            Hir::If { condition, label } => write!(f, "if {} goto l{}", condition, label),
            Hir::Print { operand } => write!(f, "print {}", operand),
            Hir::Input { dest } => write!(f, "input {}", dest),
        }
    }
}

pub struct Program {
    hir: Vec<Hir>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for hir in &self.hir {
            match hir {
                Hir::Label { .. } => {
                    writeln!(f, "{}", hir)?;
                }
                _ => {
                    writeln!(f, "\t{}", hir)?;
                }
            }
        }
        Ok(())
    }
}

struct ForInfo<'a> {
    begin_label: u32,
    end_label: u32,
    step: Option<&'a ast::Expression<'a>>,
}

pub struct HirBuilder<'a> {
    hir: Vec<Hir>,

    program: &'a ast::Program<'a>,

    var_map: HashMap<*const str, Operand>,
    expr_map: HashMap<*const ast::Expression<'a>, Operand>,

    str_map: HashMap<*const str, usize>,
    str_literals: Vec<String>,

    line_to_hir_map: HashMap<usize, usize>,

    for_stack: Vec<ForInfo<'a>>,
    goto_list: Vec<usize>,

    next_variable: u32,
    next_label: u32,
}

impl<'a> HirBuilder<'a> {
    pub fn new(program: &'a ast::Program<'a>) -> Self {
        Self {
            program,
            hir: Vec::new(),
            var_map: HashMap::new(),
            expr_map: HashMap::new(),
            line_to_hir_map: HashMap::new(),
            for_stack: Vec::new(),
            next_variable: 0,
            next_label: 0,
            str_map: HashMap::new(),
            str_literals: Vec::new(),
            goto_list: Vec::new(),
        }
    }

    pub fn build(mut self) -> (Program, Vec<String>) {
        self.program.accept(&mut self);
        let program = Program { hir: self.hir };
        (program, self.str_literals)
    }

    fn get_next_variable_id(&mut self) -> u32 {
        let id = self.next_variable;
        self.next_variable += 1;
        id
    }

    fn get_next_label(&mut self) -> u32 {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    fn insert_str_literal(&mut self, s: &str) -> u32 {
        // TODO: check overflows
        if let Some(&id) = self.str_map.get(&ptr::from_ref(s)) {
            id as u32
        } else {
            let id = self.str_literals.len();
            self.str_literals.push(s.to_string());
            self.str_map.insert(ptr::from_ref(s), id);
            id as u32
        }
    }
}

impl<'a> ExpressionVisitor<'a, Operand> for HirBuilder<'a> {
    fn visit_number_literal(&mut self, value: i32) -> Operand {
        Operand::NumberLiteral { value }
    }

    fn visit_string_literal(&mut self, content: &'a str) -> Operand {
        let index = self.insert_str_literal(content);
        Operand::IndirectNumberLiteral {
            value: index as i32,
        }
    }

    fn visit_variable(&mut self, variable: &'a str) -> Operand {
        if let Some(&id) = self.var_map.get(&ptr::from_ref(variable)) {
            id
        } else {
            let id = self.get_next_variable_id();

            let var = if variable.trim().ends_with("$") {
                Operand::IndirectVariable { id }
            } else {
                Operand::Variable { id }
            };

            self.var_map.insert(ptr::from_ref(variable), var);

            var
        }
    }

    fn visit_binary_op(
        &mut self,
        left: &'a ast::Expression<'a>,
        op: ast::BinaryOperator,
        right: &'a ast::Expression<'a>,
    ) -> Operand {
        let left_op = if let Some(&id) = self.expr_map.get(&ptr::from_ref(left)) {
            id
        } else {
            let dest = left.accept(self);
            self.expr_map.insert(ptr::from_ref(left), dest);
            dest
        };

        let right_op = if let Some(&id) = self.expr_map.get(&ptr::from_ref(right)) {
            id
        } else {
            let dest = right.accept(self);
            self.expr_map.insert(ptr::from_ref(right), dest);
            dest
        };

        // TODO: if string concatenation is allowed this has to change
        let dest_op = Operand::Variable {
            id: self.get_next_variable_id(),
        };

        let expr = match op {
            ast::BinaryOperator::Add => Expression {
                left: left_op,
                op: BinaryOperator::Add,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Sub => Expression {
                left: left_op,
                op: BinaryOperator::Sub,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Mul => Expression {
                left: left_op,
                op: BinaryOperator::Mul,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Div => Expression {
                left: left_op,
                op: BinaryOperator::Div,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::And => Expression {
                left: left_op,
                op: BinaryOperator::And,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Or => Expression {
                left: left_op,
                op: BinaryOperator::Or,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Eq => Expression {
                left: left_op,
                op: BinaryOperator::Eq,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Ne => Expression {
                left: left_op,
                op: BinaryOperator::Ne,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Lt => Expression {
                left: left_op,
                op: BinaryOperator::Lt,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Le => Expression {
                left: left_op,
                op: BinaryOperator::Le,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Gt => Expression {
                left: left_op,
                op: BinaryOperator::Gt,
                right: right_op,
                dest: dest_op,
            },
            ast::BinaryOperator::Ge => Expression {
                left: left_op,
                op: BinaryOperator::Ge,
                right: right_op,
                dest: dest_op,
            },
        };

        self.hir.push(Hir::Expression(expr));
        self.expr_map.insert(ptr::from_ref(left), dest_op);

        dest_op
    }
}

impl<'a> StatementVisitor<'a> for HirBuilder<'a> {
    fn visit_let(&mut self, variable: &'a str, expression: &ast::Expression<'a>) {
        let dest = self.visit_variable(variable);
        let src = expression.accept(self);

        self.hir.push(Hir::Copy { src, dest });
    }

    fn visit_print(&mut self, content: &'a [&'a ast::Expression<'a>]) {
        for item in content {
            let operand = item.accept(self);
            self.hir.push(Hir::Print { operand });
        }
    }

    fn visit_input(&mut self, prompt: Option<&'a ast::Expression<'a>>, variable: &'a str) {
        if let Some(prompt) = prompt {
            let prompt = prompt.accept(self);
            self.hir.push(Hir::Print { operand: prompt });
        }

        let dest = self.visit_variable(variable);
        self.hir.push(Hir::Input { dest });
    }

    fn visit_goto(&mut self, line_number: u32) {
        self.goto_list.push(self.hir.len());
        self.hir.push(Hir::Goto { label: line_number });
    }

    fn visit_for(
        &mut self,
        variable: &'a str,
        from: &ast::Expression<'a>,
        to: &ast::Expression<'a>,
        step: Option<&'a ast::Expression<'a>>,
    ) {
        let index = self.visit_variable(variable);
        let from = from.accept(self);
        self.hir.push(Hir::Copy {
            src: from,
            dest: index,
        });

        let to = to.accept(self);
        let cmp_dest = Operand::Variable {
            id: self.get_next_variable_id(),
        };

        let info = ForInfo {
            begin_label: self.get_next_label(),
            end_label: self.get_next_label(),
            step,
        };

        self.hir.push(Hir::Label {
            id: info.begin_label,
        });

        self.hir.push(Hir::Expression(Expression {
            left: index,
            op: BinaryOperator::Ge,
            right: to,
            dest: cmp_dest,
        }));

        self.hir.push(Hir::If {
            condition: cmp_dest,
            label: info.end_label,
        });

        self.for_stack.push(info);
    }

    fn visit_next(&mut self, variable: &'a str) {
        let index = self.visit_variable(variable);
        let info = self.for_stack.pop().unwrap();

        if let Some(step) = info.step {
            let step = step.accept(self);
            self.hir.push(Hir::Expression(Expression {
                left: index,
                op: BinaryOperator::Add,
                right: step,
                dest: index,
            }));
        } else {
            // Add 1 to the index variable
            self.hir.push(Hir::Expression(Expression {
                left: index,
                op: BinaryOperator::Add,
                right: Operand::NumberLiteral { value: 1 },
                dest: index,
            }));
        }

        self.hir.push(Hir::Goto {
            label: info.begin_label,
        });
        self.hir.push(Hir::Label { id: info.end_label });
    }

    fn visit_end(&mut self) {}

    fn visit_gosub(&mut self, line_number: u32) {
        self.goto_list.push(self.hir.len());
        self.hir.push(Hir::GoSub { label: line_number });
    }

    fn visit_return(&mut self) {
        self.hir.push(Hir::Return);
    }

    fn visit_if(
        &mut self,
        condition: &ast::Expression<'a>,
        then: &'a ast::Statement<'a>,
        else_: Option<&'a ast::Statement<'a>>,
    ) {
        let cond = condition.accept(self);
        let neg_cond = Operand::Variable {
            id: self.get_next_variable_id(),
        };
        self.hir.push(Hir::Expression(Expression {
            left: cond,
            op: BinaryOperator::Eq,
            right: Operand::NumberLiteral { value: 0 },
            dest: neg_cond,
        }));

        let label = self.get_next_label();

        self.hir.push(Hir::If {
            condition: neg_cond,
            label,
        });

        then.accept(self);

        self.hir.push(Hir::Label { id: label });

        if let Some(else_) = else_ {
            else_.accept(self);
        }
    }

    fn visit_seq(&mut self, statements: &'a [ast::Statement<'a>]) {
        for stmt in statements {
            stmt.accept(self);
        }
    }
}

impl<'a> ProgramVisitor<'a> for HirBuilder<'a> {
    fn visit_program(&mut self, program: &'a ast::Program<'a>) {
        for (&line_number, stmt) in program.iter() {
            self.line_to_hir_map
                .insert(line_number as usize, self.hir.len());
            stmt.accept(self);
        }

        let mut offset = 0;

        while let Some(og_goto) = self.goto_list.pop() {
            let goto = og_goto + offset;
            let line = if let Hir::Goto { label: line } = &self.hir[goto] {
                *line as usize
            } else if let Hir::GoSub { label: line } = &self.hir[goto] {
                *line as usize
            } else {
                unreachable!("Invalid goto position");
            };

            // Add label before jump position
            let new_label_pos = *self.line_to_hir_map.get(&line).unwrap() + offset;

            // check there is already a label
            // TODO: ugly as fuck
            let new_label = if new_label_pos > 1 {
                if let Hir::Label { id } = &self.hir[new_label_pos - 1] {
                    *id
                } else {
                    let new_label = self.get_next_label();

                    self.hir.insert(new_label_pos, Hir::Label { id: new_label });

                    offset += 1;

                    new_label
                }
            } else {
                let new_label = self.get_next_label();

                self.hir.insert(new_label_pos, Hir::Label { id: new_label });

                offset += 1;

                new_label
            };

            let goto = og_goto + offset;

            if let Hir::Goto { .. } = &self.hir[goto] {
                self.hir[goto] = Hir::Goto { label: new_label };
            } else if let Hir::GoSub { .. } = &self.hir[goto] {
                self.hir[goto] = Hir::GoSub { label: new_label };
            } else {
                unreachable!("Invalid goto position");
            }
        }
    }
}
