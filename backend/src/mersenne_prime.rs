use primes::is_prime;

#[allow(dead_code)]
fn find_next_mersenne_prime(earlier_prime: u32) {
    let mut p = earlier_prime + 1;
    while !lucas_lehmer_test(p) {
        p += 1;
    }
}

fn lucas_lehmer_test(n: u32) -> bool {
    if n == 2 {
        return true;
    }

    if !is_prime(n.into()) {
        return false;
    }

    let m = u32::pow(2, n) - 1;
    let mut s = 4;

    for _ in 2..n {
        let square = s * s;
        s = (square & m) + (square >> n);
        if s >= m {
            s -= m;
        }

        s -= 2;
    }

    return s == 0;
}
