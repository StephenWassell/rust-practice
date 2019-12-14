use queens;

#[test]
fn test_size_1() {
	assert_eq!(queens::queens(0, true), 1);
}

#[test]
fn test_size_8() {
	assert_eq!(queens::queens(8, true), 92);
}
