use super::*;

#[test]
fn compact_route_chain_shortens_long_routes() {
    assert_eq!(
        compact_route_chain("M→$→M→?→M→?→R→M→T→M→E→R→M→M→R"),
        "M→$→M→?→M→…→R→M→M→R"
    );
}

#[test]
fn compact_route_chain_short() {
    assert_eq!(compact_route_chain("M→R→E→R"), "M→R→E→R");
}

#[test]
fn compact_route_chain_exactly_ten_parts() {
    let chain = "M→R→M→E→?→$→M→R→M→T";
    assert_eq!(compact_route_chain(chain), chain);
}

#[test]
fn compact_route_chain_eleven_parts() {
    let chain = "M→R→M→E→?→$→M→R→M→T→R";
    let result = compact_route_chain(chain);
    assert_eq!(result, "M→R→M→E→?→…→R→M→T→R");
}
