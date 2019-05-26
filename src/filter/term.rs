use super::cmp::*;
use super::value_filter::ValueFilterKey;
use super::value_manager::*;
use std::cell::RefCell;
use select::path_map::PathMap;
use std::sync::Arc;

#[derive(Debug)]
pub enum TermContext {
    Constants(ExprTerm),
    Json(Option<ValueFilterKey>, ValueManager),
}

impl TermContext {
    fn cmp<F: PrivCmp + IntoType>(&mut self, other: &mut TermContext, cmp_fn: F, default: bool) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        trace!("const-const");
                        TermContext::Constants(ExprTerm::Bool(et.cmp(oet, cmp_fn, default)))
                    }
                    TermContext::Json(ref mut key, ref mut v) => {
                        trace!("const-json");
                        TermContext::Json(None, v.get_compare_with(key, et, cmp_fn, true))
                    }
                }
            }
            TermContext::Json(key, v) => {
                match other {
                    TermContext::Json(key_other, ov) => {
                        trace!("json-json");

                        fn is_json(t: &TermContext) -> bool {
                            match t {
                                TermContext::Json(_, _) => true,
                                _ => false
                            }
                        }

                        let mut c = v.into_term(key);
                        let mut oc = ov.into_term(key_other);
                        if is_json(&c) && is_json(&oc) {
                            v.cmp(&ov, cmp_fn.into_type())
                        } else {
                            c.cmp(&mut oc, cmp_fn, default)
                        }
                    }
                    TermContext::Constants(et) => {
                        trace!("json-const");
                        TermContext::Json(None, v.get_compare_with(key, et, cmp_fn, false))
                    }
                }
            }
        }
    }

    fn cmp_cond(&self, other: &TermContext, cmp_cond_type: CmpCondType, path_map: Arc<RefCell<PathMap>>) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        match cmp_cond_type {
                            CmpCondType::Or => {
                                TermContext::Constants(ExprTerm::Bool(et.cmp(oet, CmpOr, false)))
                            }
                            CmpCondType::And => {
                                TermContext::Constants(ExprTerm::Bool(et.cmp(oet, CmpAnd, false)))
                            }
                        }
                    }
                    TermContext::Json(_, v) => {
                        TermContext::Json(None, ValueManager::new(v.get_val().clone(), false, path_map))
                    }
                }
            }
            TermContext::Json(_, v) => {
                match other {
                    TermContext::Json(_, ov) => {
                        match cmp_cond_type {
                            CmpCondType::Or => TermContext::Json(None, v.union(ov)),
                            CmpCondType::And => TermContext::Json(None, v.intersect(ov)),
                        }
                    }
                    _ => {
                        TermContext::Json(None, ValueManager::new(v.get_val().clone(), false, path_map))
                    }
                }
            }
        }
    }

    pub fn eq(&mut self, other: &mut TermContext) -> TermContext {
        trace!("eq");
        self.cmp(other, CmpEq, false)
    }

    pub fn ne(&mut self, other: &mut TermContext) -> TermContext {
        trace!("ne");
        self.cmp(other, CmpNe, true)
    }

    pub fn gt(&mut self, other: &mut TermContext) -> TermContext {
        trace!("gt");
        self.cmp(other, CmpGt, false)
    }

    pub fn ge(&mut self, other: &mut TermContext) -> TermContext {
        trace!("ge");
        self.cmp(other, CmpGe, false)
    }

    pub fn lt(&mut self, other: &mut TermContext) -> TermContext {
        trace!("lt");
        self.cmp(other, CmpLt, false)
    }

    pub fn le(&mut self, other: &mut TermContext) -> TermContext {
        trace!("le");
        self.cmp(other, CmpLe, false)
    }

    pub fn and(&mut self, other: &mut TermContext, path_map: Arc<RefCell<PathMap>>) -> TermContext {
        self.cmp_cond(other, CmpCondType::And, path_map)
    }

    pub fn or(&mut self, other: &mut TermContext, path_map: Arc<RefCell<PathMap>>) -> TermContext {
        self.cmp_cond(other, CmpCondType::Or, path_map)
    }
}


#[derive(Debug)]
pub enum ExprTerm {
    String(String),
    Number(f64),
    Bool(bool),
}

impl ExprTerm {
    fn cmp<F: PrivCmp>(&self, other: &ExprTerm, cmp_fn: F, default: bool) -> bool {
        match self {
            ExprTerm::Bool(v1) => match other {
                ExprTerm::Bool(v2) => cmp_fn.cmp_bool(v1, v2),
                _ => default
            }
            ExprTerm::Number(v1) => match other {
                ExprTerm::Number(v2) => cmp_fn.cmp_f64(v1, v2),
                _ => default
            }
            ExprTerm::String(v1) => match other {
                ExprTerm::String(v2) => cmp_fn.cmp_string(v1, v2),
                _ => default
            }
        }
    }
}
