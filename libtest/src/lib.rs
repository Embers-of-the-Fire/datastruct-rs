#[cfg(test)]
mod test_ops;

use datastruct::DataStruct;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DataStruct)]
#[dstruct(default, const, set)]
struct DevTest {
    #[dfield(default = "10")]
    field1: u8,
    #[dfield(default = "10")]
    field2: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, DataStruct)]
#[dstruct(default, get)]
struct RichDevTest {
    #[dfield(default = "vec![]", get = "full")]
    pub vec: Vec<u8>,
    #[dfield(default = "fn_default()")]
    pub cnt: usize,
}

const fn fn_default() -> usize {
    10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DataStruct)]
#[dstruct(partial)]
struct NotAllDefault {
    #[dfield(map)]
    value: u8,
    #[dfield(default = "1", do_with)]
    value_default: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DataStruct)]
#[dstruct(default, const)]
struct SelfReference {
    #[dfield(default = "val2 + 1")]
    val1: u8,
    #[dfield(default = "10")]
    #[dfield(seq = -1)]
    val2: u8,
}

#[derive(Clone, Copy, DataStruct)]
#[dstruct(debug)]
struct Debuggable {
    val1: u8,
    #[dfield(no_debug)]
    val2: u8,
}

#[derive(Debug, Clone, Copy, DataStruct)]
#[dstruct(cmp(peq, eq, pord, ord))]
struct PartlyEq {
    #[dfield(cmp(ord = true, pord = true))]
    can_eq: u8,
    #[dfield(cmp(eq = false))]
    do_not_check_eq: u8,
}

#[test]
fn test_attr() {
    use datastruct::{ConstDataStruct, DataStruct};
    assert_eq!(
        DevTest {
            field1: 10,
            field2: 10,
        },
        DevTest::data_default()
    );
    assert_eq!(
        DevTest {
            field1: 10,
            field2: 10,
        },
        DevTest::DEFAULT
    );
    let default = DevTest::data_default().with_field1(5);
    assert_eq!(
        default,
        DevTest {
            field1: 5,
            field2: 10
        }
    );

    let mut rich_default = RichDevTest::data_default();
    assert_eq!(
        RichDevTest {
            vec: vec![],
            cnt: 10,
        },
        rich_default
    );
    rich_default.vec.push(10);
    assert_eq!(
        RichDevTest {
            vec: vec![10],
            cnt: 10,
        },
        rich_default
    );
    assert_eq!(&vec![10], rich_default.vec());
    assert_eq!(vec![10], rich_default.clone().get_vec());

    assert_eq!(
        NotAllDefault {
            value: 5,
            value_default: 1
        },
        NotAllDefault::partial_default(5)
    );
    let mut not_all_default = NotAllDefault::partial_default(5).map_value(|d| d + 1);
    assert_eq!(
        NotAllDefault {
            value: 6,
            value_default: 1
        },
        not_all_default
    );
    not_all_default.do_with_value_default(|v| *v += 1);
    assert_eq!(
        NotAllDefault {
            value: 6,
            value_default: 2
        },
        not_all_default
    );

    assert_eq!(
        SelfReference { val1: 11, val2: 10 },
        SelfReference::data_default()
    );
    assert_eq!(SelfReference { val1: 11, val2: 10 }, SelfReference::DEFAULT);

    assert_eq!(
        "Debuggable { val1: 10 }",
        format!("{:?}", Debuggable { val1: 10, val2: 10 })
    );

    assert_eq!(
        PartlyEq {
            can_eq: 10,
            do_not_check_eq: 5
        },
        PartlyEq {
            can_eq: 10,
            do_not_check_eq: 0
        }
    );
    assert_ne!(
        PartlyEq {
            can_eq: 10,
            do_not_check_eq: 5
        },
        PartlyEq {
            can_eq: 3,
            do_not_check_eq: 5
        }
    );
}

#[test]
fn test_adv() {
    use datastruct::DataStruct;
    #[derive(DataStruct)]
    #[dstruct(debug)]
    struct Person {
        age: u8,
        name: String,
        #[dfield(no_debug)]
        private_key: u32,
    }
    let person = Person {
        age: 22,
        name: "James".to_string(),
        private_key: 42,
    };
    println!("{:#?}", person);
}
