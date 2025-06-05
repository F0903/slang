? comment

fn outer() {
    let x = "outside"
    fn inner() {
        print_line(x)
    }
    inner()
}

outer()