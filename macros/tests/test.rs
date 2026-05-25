use macros::New;

#[test]
fn test_macro() {
    #[derive(New, PartialEq, Debug)]
    struct Hey {
        a: u8,
        b: String,
    }

    assert_eq!(
        Hey::new(123, "123".to_string()),
        Hey {
            a: 123,
            b: "123".to_string()
        }
    );
}
