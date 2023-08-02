use std::{
    fmt::Display,
    hash::Hash,
    ops::{Add, Mul, Neg},
};

use crate::{field::Field, pairing::G1};

use super::types::{f101, F101};

#[allow(non_snake_case)]
pub fn g1f(x: u64, y: u64) -> G1P {
    G1P::new(F101::from(x), F101::from(y))
}

/// A point in the $y^2+x^3+3$ curve, on the $\mathbb{F}_{101}$ field.
/// The generator $g=(1,2)$ generates a subgroup of order 17: $17g=g$
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct G1P {
    /// x coordinate
    pub x: F101,
    /// y coordinate
    pub y: F101,
    /// if point is at infinity
    pub infinite: bool,
}

/// An elliptic curve is an equation of the form $y^2=x^3+ax+b$ where $a$, $b$, $x$, and $y$
/// are elements of some field. (In a cryptographic setting the field will be a finite field.)
/// Any $(x,y)$ pairs satisfying the equation are points on the curve.
///
/// We will use an elliptic curve over the field $\mathbb{F}_{101}$ , which makes hand computation easy.
///
/// The elliptic curve $y^2 = x^3 +3$ is a commonly-used elliptic curve equation, and $(1,2)$ is an easy-to-find
/// point on that curve we can use as a generator. In fact, the alt_bn128 curve that is implemented on Ethereum
/// uses this same curve equation and generator, but over a much larger field.
///
/// Point addition:
/// for $P_r = P_p +  P_q$ , $x_r = \lambda^2 - x_p - x_q$ $y_r = \lambda(x_p - x_r) - y_p$
/// where $\\lambda = \\frac{y_q - y_p}{x_q - x_p}$
///
/// Point doubling (This doubling formula only works for curves with a=0, like ours)
/// for $P=(x,y)$, $2P=(m^2-2x, m(3x-m^2)-y)$ where $m=\\frac{3x^2}{2y}$
///
/// Point inversion:
/// for $P=(x,y)$,  $-P=(x,-y)$
///
/// Elliptic curves also have an abstract "point at infinity" which serves as the group identity. For more on elliptic curve arithmetic, check out [this post from Nick Sullivan](https://blog.cloudflare.com/a-relatively-easy-to-understand-primer-on-elliptic-curve-cryptography/)
///
impl G1 for G1P {
    type F = F101;
    type S = F101;

    /// Creates a new point at given $(x,y)$
    fn new(x: Self::F, y: Self::F) -> Self {
        G1P {
            x,
            y,
            infinite: false,
        }
    }
    /// Checks if the coordinates are on the curve, so $y^2 = x^3 +3$
    fn in_curve(&self) -> bool {
        self.y.pow(2) == self.x.pow(3) + f101(3u64)
    }
    /// Checks if the point is at infinity
    fn is_identity(&self) -> bool {
        self.infinite
    }
    /// Returns the generator $g=(1,2)$
    fn generator() -> Self {
        G1P {
            x: f101(1u64),
            y: f101(2u64),
            infinite: false,
        }
    }
    /// Returns the size of the subgroup generated by generator $g=(1,2)$
    fn generator_subgroup_size() -> Self::F {
        f101(17u64)
    }
    /// Returns the point at infinity
    fn identity() -> Self {
        G1P {
            x: Self::F::zero(),
            y: Self::F::zero(),
            infinite: true,
        }
    }
    fn x(&self) -> &Self::F {
        &self.x
    }
    fn y(&self) -> &Self::F {
        &self.y
    }
}

impl Display for G1P {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.infinite {
            write!(f, "infinite")
        } else {
            write!(f, "({},{})", self.x, self.y)
        }
    }
}

impl Neg for G1P {
    type Output = G1P;
    fn neg(self) -> Self::Output {
        if self.infinite {
            self
        } else {
            G1P::new(self.x, -self.y)
        }
    }
}

impl Add for G1P {
    type Output = G1P;
    fn add(self, rhs: G1P) -> Self {
        if self.infinite {
            rhs
        } else if rhs.infinite {
            self
        } else if self == -rhs {
            G1P::identity()
        } else if self == rhs {
            let two = f101(2);
            let three = f101(3);
            let m = ((three * self.x.pow(2)) / (two * self.y)).unwrap();
            G1P::new(
                m * m - two * self.x,
                m * (three * self.x - m.pow(2)) - self.y,
            )
        } else {
            // https://en.wikipedia.org/wiki/Elliptic_curve_point_multiplication#G1P_addition
            let lambda = ((rhs.y - self.y) / (rhs.x - self.x))
                .unwrap_or_else(|| panic!("cannot add {}+{}", self, rhs));
            let x = lambda.pow(2) - self.x - rhs.x;
            G1P::new(x, lambda * (self.x - x) - self.y)
        }
    }
}

impl Mul<F101> for G1P {
    type Output = G1P;
    fn mul(self, rhs: F101) -> Self::Output {
        let mut rhs = rhs.as_u64();
        if rhs == 0 || self.is_identity() {
            return G1P::identity();
        }
        let mut result = None;
        let mut base = self;
        while rhs > 0 {
            if rhs % 2 == 1 {
                result = Some(if let Some(result) = result {
                    result + base
                } else {
                    base
                })
            }
            rhs >>= 1;
            base = base + base;
        }
        result.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_subgroups() {
        // add all points that are in the curve
        let mut points = Vec::new();
        for x in 0..101 {
            for y in 0..101 {
                let p = G1P::new(f101(x), f101(y));
                if p.in_curve() {
                    points.push(p);
                }
            }
        }

        // find subgroups
        let mut subgroups = std::collections::HashMap::new();
        while !points.is_empty() {
            let mut in_subgroup = Vec::new();

            // pick one element as a generator
            let g = points.pop().unwrap();
            in_subgroup.push(g);

            // find all m * g != g
            let mut mul = 2;
            let mut gmul = g * f101(mul);
            while g != gmul {
                in_subgroup.push(gmul);
                mul += 1;
                gmul = g * f101(mul);
            }
            in_subgroup.sort();
            subgroups.insert(g, in_subgroup);
        }

        // find unique subgroups
        let mut duplicates = Vec::new();
        for (g, e) in &subgroups {
            if !duplicates.contains(g) {
                subgroups
                    .iter()
                    .filter(|(g1, e1)| g1 != &g && e1 == &e)
                    .for_each(|(g, _)| duplicates.push(*g));
            }
        }

        // remove duplicates
        subgroups.retain(|g, _| !duplicates.contains(g));

        for (g, e) in &subgroups {
            println!("{} {}", g, e.len());
            if e.len() < 20 {
                for n in e {
                    println!("  {}", n);
                }
            }
        }
    }

    #[test]
    fn test_g1_vectors() {
        let g = G1P::generator();
        let two_g = g + g;
        let four_g = two_g + two_g;
        let eight_g = four_g + four_g;
        let sixteen_g = eight_g + eight_g;

        assert_eq!(g1f(1, 99), -g);
        assert_eq!(g1f(68, 74), two_g);
        assert_eq!(g1f(68, 27), -two_g);
        assert_eq!(g1f(65, 98), four_g);
        assert_eq!(g1f(65, 3), -four_g);
        assert_eq!(g1f(18, 49), eight_g);
        assert_eq!(g1f(18, 52), -eight_g);
        assert_eq!(g1f(1, 99), sixteen_g);
        assert_eq!(g1f(1, 2), -sixteen_g);

        // since g = -16 g, this subgroup has order 17

        assert_eq!(g1f(26, 45), two_g + g);
        assert_eq!(g1f(12, 32), four_g + g);
        assert_eq!(g1f(18, 52), eight_g + g);
        assert_eq!(four_g + two_g, two_g + four_g);

        assert_eq!(g * f101(1), g);
        assert_eq!(g * f101(2), g + g);
        assert_eq!(g * f101(6), g + g + g + g + g + g);
    }
}
