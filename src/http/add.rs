struct Request {
    Scalars Vec<uint>
}

struct Response {
    Scalars Vec<ScalarResult>,
}

struct ScalarResult {
    Scalar uint,
    Added bool,
}
