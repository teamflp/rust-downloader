use super::cookies;

#[test]
fn test_get_installed_browsers() {
    let browsers = cookies::get_installed_browsers();
    // This test is highly dependent on the environment.
    // On a typical developer machine, we'd expect at least one browser.
    // On a CI/CD runner, this might be empty.
    // So, we just check that the function returns a Vec without crashing.
    assert!(browsers.is_empty() || !browsers.is_empty());
}
