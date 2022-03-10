use primes::is_prime;

pub(crate) fn find_next_mersenne_prime(earlier_prime: u32) -> u32 {
    let mut p = earlier_prime + 1;
    while !lucas_lehmer_test(p) {
        p += 1;
    }
    p
}

fn lucas_lehmer_test(n: u32) -> bool {
    if n == 2 {
        return true;
    }

    if !is_prime(n.into()) {
        return false;
    }

    let mut s: u64 = 4;
    let m: u64 = u64::from(u32::pow(2, n) - 1);

    for _ in 2..n {
        s = ((s * s) - 2) % m;
    }

    return s == 0;
}
