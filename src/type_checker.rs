use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;

macro_rules! issue {
    ($rec:expr, $loc: expr, $err: expr) => {
        $rec.errors.push(Error::new($loc, $err));
    };
}

#[derive(Debug, Copy, Clone)]
pub enum Loop {
    While(*const While),
    For(*const For),
    Foreach(*const Foreach),
}

pub struct TypeChecker {
    errors: Vec<Error>,
    scopes: ScopeStack,
    loops: Vec<Loop>,
    current_method: *const MethodDef,
}

impl TypeChecker {
    //    fn check_binary(&mut self, left: &mut Expr, right: &mut Expr, op: Operator) -> Type {
//        self.visit_expr(left);
//        self.visit_expr(right);
////        if left.get
//    }
    fn check_bool(&mut self, expr: &mut Expr) {
        self.visit_expr(expr);
        let t = expr.get_type();
        if !require_type(t, &BOOL) {
            issue!(self, expr.get_loc(), TestNotBool);
        }
    }
}

fn require_type(type_: &SemanticType, target: &SemanticType) -> bool {
    type_ == &ERROR || type_ == target
}

impl Visitor for TypeChecker {
    fn visit_program(&mut self, program: &mut Program) {
        self.scopes.open(&mut program.scope);
        for class_def in &mut program.classes { self.visit_class_def(class_def); }
        self.scopes.close();
    }

    fn visit_class_def(&mut self, class_def: &mut ClassDef) {
        self.scopes.open(&mut class_def.scope);
        for field_def in &mut class_def.fields { self.visit_field_def(field_def) }
        self.scopes.close();
    }

    fn visit_method_def(&mut self, method_def_: &mut MethodDef) {
        self.current_method = method_def_;
        self.scopes.open(&mut method_def_.scope);
        self.visit_block(&mut method_def_.body);
        self.scopes.close();
    }

    fn visit_block(&mut self, block: &mut Block) {
        self.scopes.open(&mut block.scope);
        for statement in &mut block.statements { self.visit_statement(statement); }
        self.scopes.close();
    }

    fn visit_while(&mut self, while_: &mut While) {
        self.check_bool(&mut while_.cond);
        self.loops.push(Loop::While(while_));
        self.visit_statement(&mut while_.body);
        self.loops.pop();
    }

    fn visit_unary(&mut self, unary: &mut Unary) {
        self.visit_expr(&mut unary.opr);
        let opr = unary.opr.get_type();
        match unary.opt {
            Operator::Neg => {
                if require_type(opr, &INT) {
                    unary.type_ = INT;
                } else {
                    issue!(self, unary.loc, IncompatibleUnary { opt: "-", type_: opr.to_string() });
                    unary.type_ = ERROR;
                }
            }
            Operator::Not => {
                if !require_type(opr, &BOOL) {
                    issue!(self, unary.loc, IncompatibleUnary { opt: "!", type_: opr.to_string() });
                    unary.type_ = ERROR;
                }
                // no matter error or not, set type to bool
                unary.type_ = BOOL;
            }
            _ => unreachable!(),
        }
    }

    fn visit_binary(&mut self, binary: &mut Binary) {
        self.visit_expr(&mut binary.left);
        self.visit_expr(&mut binary.right);
        let (left, right) = (&*binary.left, &*binary.right);
        let (left_t, right_t) = (left.get_type(), right.get_type());
        if left_t == &ERROR || right_t == &ERROR {
            match binary.opt {
                Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => binary.type_ = left_t.clone(),
                Operator::Mod => binary.type_ = INT,
                Operator::Repeat | Operator::Concat => unimplemented!(),
                _ => binary.type_ = BOOL,
            }
            return;
        }
        // TODO move repeat & concat out from binary operator(both in java & rust version)
        if {
            !match binary.opt {
                Operator::Add | Operator::Sub | Operator::Mul | Operator::Div | Operator::Mod => {
                    binary.type_ = left_t.clone();
                    left_t == &INT && right_t == &INT
                }
                Operator::Lt | Operator::Le | Operator::Gt | Operator::Ge => {
                    binary.type_ = BOOL;
                    left_t == &INT && right_t == &INT
                }
                Operator::Eq | Operator::Ne => {
                    binary.type_ = BOOL;
                    left_t == right_t
                }
                Operator::And | Operator::Or => {
                    binary.type_ = BOOL;
                    left_t == &BOOL && right_t == &BOOL
                }
                Operator::Repeat | Operator::Concat => unimplemented!(),
                _ => unreachable!(),
            }
        } {
            issue!(self, binary.loc, IncompatibleBinary {
                left_type: left_t.to_string(),
                opt: binary.opt.to_str(),
                right_type: right_t.to_string(),
            });
        }
    }
}