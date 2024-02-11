use datastruct::DataStruct;

#[derive(Debug, Clone, Copy, DataStruct)]
#[dstruct(ops(add, sub, div), cmp(eq, peq))]
struct CanOps {
    #[dfield(ops(sub = "ignore"))]
    can_add: i8,
    #[dfield(ops(add = "ignore"))]
    can_sub: i8,
}

#[derive(Debug, Clone, Copy, DataStruct)]
#[dstruct(ops(add = "both"), cmp(eq, peq))]
struct CanOpsAssign {
    can_add: i8,
    #[dfield(ops(add_assign = "ignore"))]
    no_add: i8,
    #[dfield(ops(add_assign = "std::cmp::max($self.max, $rhs.max)"))]
    max: i8,
    #[dfield(ops(add_assign = "std::cmp::min($self.min, $rhs.min)"))]
    min: i8,
}

#[test]
fn test_ops() {
    let add1 = CanOps {
        can_add: 0,
        can_sub: 10,
    };
    let add2 = CanOps {
        can_add: 10,
        can_sub: 0,
    };
    assert_eq!(
        CanOps {
            can_add: 10,
            can_sub: 10
        },
        add1 + add2
    );
    assert_eq!(
        CanOps {
            can_add: 0,
            can_sub: 10
        },
        add1 - add2
    );
    assert_eq!(
        CanOps {
            can_add: 10,
            can_sub: -10
        },
        add2 - add1
    );

    let mut add_base = CanOpsAssign {
        can_add: 10,
        no_add: 10,
        max: 10,
        min: 15,
    };
    add_base += CanOpsAssign {
        can_add: 10,
        no_add: 10,
        max: 15,
        min: 10,
    };
    assert_eq!(
        add_base,
        CanOpsAssign {
            can_add: 20,
            no_add: 10,
            max: 15,
            min: 10
        }
    );
}
