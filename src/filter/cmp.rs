pub enum CmpType {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

pub enum CmpCondType {
    And,
    Or,
}

pub trait PrivCmp {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool;

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool;

    fn cmp_string(&self, v1: &String, v2: &String) -> bool;
}

pub trait IntoType {
    fn into_type(&self) -> CmpType;
}

pub struct CmpEq;

impl PrivCmp for CmpEq {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 == v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 == v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 == v2
    }
}

impl IntoType for CmpEq {
    fn into_type(&self) -> CmpType {
        CmpType::Eq
    }
}

pub struct CmpNe;

impl PrivCmp for CmpNe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 != v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 != v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 != v2
    }
}

impl IntoType for CmpNe {
    fn into_type(&self) -> CmpType {
        CmpType::Ne
    }
}

pub struct CmpGt;

impl PrivCmp for CmpGt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 > v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 > v2
    }
}

impl IntoType for CmpGt {
    fn into_type(&self) -> CmpType {
        CmpType::Gt
    }
}

pub struct CmpGe;

impl PrivCmp for CmpGe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 >= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 >= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 >= v2
    }
}

impl IntoType for CmpGe {
    fn into_type(&self) -> CmpType {
        CmpType::Ge
    }
}

pub struct CmpLt;

impl PrivCmp for CmpLt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 < v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 < v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 < v2
    }
}

impl IntoType for CmpLt {
    fn into_type(&self) -> CmpType {
        CmpType::Lt
    }
}

pub struct CmpLe;

impl PrivCmp for CmpLe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 <= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 <= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 <= v2
    }
}

impl IntoType for CmpLe {
    fn into_type(&self) -> CmpType {
        CmpType::Le
    }
}

pub struct CmpAnd;

impl PrivCmp for CmpAnd {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 && *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 && v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() && !v2.is_empty()
    }
}

pub struct CmpOr;

impl PrivCmp for CmpOr {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 || *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 || v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() || !v2.is_empty()
    }
}