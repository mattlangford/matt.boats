---
date: 2025-09-03
---

# Floating Point Numbers
_Float like a butterfly, sting like a bee_

Floating point numbers are such a common occurrence while programming that they're worth understanding at a fundamental level. Here I will go over what each of the bits of the float represent.

The [IEEE-754 Standard](https://www-users.cse.umn.edu/~vinals/tspot_files/phys4041/2020/IEEE%20Standard%20754-2019.pdf) defines the rules for how floating point numbers interact, but here I'd like to show some specific examples with a bit of math.
According to the standard, floats are made up of 16, 32, 64, or 128 bits.
There are a few constants defined by the standard which will come in handy later.
![table](table.png)

Here is the raw bit representation of a 32 bit float
![Overview](overview.png)

Let's dig into what each group means!

## Sign bit

This is probably the most obvious bit. It can either be 0 representing a positive number, or a 1 representing a negative one.

## Exponent bits

From the table, these 5, 8, 11, or 15 bits (for 32, 64, 128 bit floats respectively) represent the _biased_ exponent of the float.

We can break down the biased exponent into a set of bits ($e_0, e_1, e_2, \cdots$) and compute its raw value:

$$
b_{\text{exponent}} = e_0\,2^{0} + e_1\,2^{1} + \cdots + e_N\,2^{N}
$$


Then the true unbiased exponent value can be computed by subtracting off the bias from the table (15, 127, 1023, or 16383), like this for 32 bits:

$$
\text{exponent} = 2^{\, (b_{\text{exp}} - 127)}
$$

There are two special exponents:

 * All bits are 0 - this represents a _subnormal_ float, which we'll largely ignore
 * All bits are 1 - this either represents a NaN or an inf (see [special values](#special-values))

Taking a 32 bit float for example, this gives us a range of 

$$
\left[\, 2^{1-127},\; 2^{254-127} \,\right]
\approx \left[\, 1.17549435\times 10^{-38},\; 1.70141183\times 10^{38} \,\right]
$$

which is absolutely enormous! For floats that represent meters, this ranges from smaller than the planck length (`~10^-35 meters`) to much larger than the Milky Way Galaxy (`~10^20 meters`).

## Mantissa bits

The exponent gives the floating point number its magnitude, but the mantissa gives the float its precision. 

The remaining 10, 23, 52, or 112 bits are called the mantissa.
The mantissa represents the fractional part of the float.
We can likewise break this down into bits ($m_0, m_1, m_2, \cdots$) and compute the mantissa value as:

$$
\text{mantissa} = 2^{0} + m_0\,2^{-1} + m_1\,2^{-2} + \cdots + m_N\,2^{-N-1}
$$

You'll notice there isn't a bit for the 2^0 term, this is intentional. For _normal_ floats, we can add on a 2^0 for free!
This win is twofold: we get some extra precision without needing the extra space, and hardware implementations can specialize and assume a leading 1.
_Subnormal_ floats are a special case without the leading 1, but I'll ignore them here.


For 32 bit floats (including the leading 1), the mantissa can range between

$$
\left[\,1,\; 1 + \sum_{i=1}^{23} 2^{-i} \,\right]
\approx \left[\, 1.0,\; 1.9999998807907104\,\right]
$$


## Putting it all together
Combining terms from the previous sections, we can come up with the formula for a _normal_ float:

$$
\text{value} = \text{sign} \ \times\  2^{\sum_{i=0}^{E} e_{i} \cdot 2^{i} - bias} \ \times \ (2^{0} \plus \sum_{i=0}^{M} m_{i} \cdot 2^{-(i + 1)})
$$

Or with all the bit math removed

$$
\text{value} \;=\; \text{sign}\,\cdot\, 2^{exponent}\,\cdot\, \mathrm{mantissa}
$$

I'll be including screenshots for 32 bit floats from the [IEEE-754 Floating Point calculator](https://www.h-schmidt.net/FloatConverter/IEEE754.html) which is a really helpful tool to understand how they work in the wild...

### 1.25
![1.255](normal.png)

|          | Bit Value | Actual Value  |
| ---------| --------- | ------------- |
| Sign     |    `0`    |     `1`       |
| Exponent | `2^0 + 2^1 + ... 2^6 = 127` | `2^(127 - 127) = 2^0 = 1` |
| Mantissa | `2^-2 = 0.25`  | `1.25` |
| Value    | `1 * 2^(2^0 + ... - 127) * (1 + 0.25)` | `1.25` |

### Pi
![Pi](pi.png)

|          | Bit Value | Actual Value  |
| ---------| --------- | ------------- |
| Sign     |    `0`    |     `1`       |
| Exponent | `2^7 = 128` | `2^(128 - 127) = 2^1 = 2` |
| Mantissa | `2^-1 + 2^-4 + ... + 2^-23 = 0.570...`  | `1.57...` |
| Value    | `1 * 2^(2^7 - 127) * (1 + 2^-1 + ... + 2^-23)` | `3.1415...` |

### 12345.6789
![12345.6789](big.png)

|          | Bit Value | Actual Value  |
| ---------| --------- | ------------- |
| Sign     |    `1`    |     `-1`      |
| Exponent | `2^2 + 2^3 + 2^7 = 140` | `2^(140 - 127) = 2^13 = 8192` |
| Mantissa | `2^-1 + 2^-8 + ... + 2^-23 = 0.507...`  | `1.507...` |
| Value    | `-1 * 2^(2^2 + ... - 127) * (1 + 2^-1 + ... + 2^-23)` | `12345.6789` |


## Caveats

It's important to keep in mind that floating point numbers arne't evenly spaced.
At the lower end, adding 1.0 to the mantissa represents a step of ~2^-126, whereas at the upper end it is a massive ~2^127.
This dramatic change can produce odd and surprising results when doing math on floating point numbers where precision in the operation is lost.

One place this comes up is when dealing with things like timestamps. If you count seconds with a 32 bit float, after counting for one day, the lowest step you can make is only 10 milliseconds.

## Special values

Here is a table of the largest and smallest (_normal_) floats of the different sizes

|          | Min Value | Max Value |
| ---------| -------   | --------  |
| Float16  | ~10^-5    |  ~10^4    |
| Float32  | ~10^-38   |  ~10^38   |
| Float64  | ~10^-308  |  ~10^308  |
| Float128 | ~10^-4932 |  ~10^4932 |


The standard defines a couple (really a few million) special values. As I alluded to earlier, when all bits are set in the exponent, the float represents either infinity, or [NaN](https://en.wikipedia.org/wiki/NaN).
These values can be "stumbled upon" when doing math - by dividing by zero, taking the square root of a negative number, or a handful of other dubious operations.

When the exponent has all bits set but none are set in the mantissa, the number is either positive or negative infinity. When _any_ bits are set in the mantissa we have a NaN value. This gives 2^24 (2^23 mantissa * 2 sign bit) total NaN values - that's a lot of nothing!
