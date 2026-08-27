pub fn libc_dependency_fixture() -> usize {
    std::mem::size_of::<libc::size_t>()
}
