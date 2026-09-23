# DSP Notes

## First-Order

### Low-Pass (LPF)

+ normalized analog BW filter:
  $$
    H_n(s) = \frac{1}{B_n(s)}
  $$

+ first-order, $n=1$:
  $$
    H(s) = \frac{1}{s+1}
  $$

+ cutoff angular frequency:
  $$
    \omega_c = 2 \pi f_c
  $$

+ wrapped-cutoff angular frequency:
  $$
    \Omega_c = 2 f_s \tan (\pi \frac{f_c}{f_s})
  $$

+ substitute $s \rarr s/\Omega_c$:
  $$
    \begin{align*}
      H(s) &= \frac{1}{s/\Omega_c + 1} \\
          &= \frac{\Omega_c}{s + \Omega_c}
    \end{align*}
  $$

+ bilinear transform:
  $$
    s \larr 2 f_s \frac{z - 1}{z + 1}
  $$

+ apply:
  $$
    \begin{align*}
      H(s) &= \frac{\Omega_c}{s + \Omega_c} \\
      H(z) &= \frac{\Omega_c}{2 f_s \frac{z - 1}{z + 1} + \Omega_c} \\
      &= \frac{\Omega_c (z + 1)}{2 f_s (z - 1) + \Omega_c (z + 1)} \\
      &= \frac{z\Omega_c + \Omega_c}{2 f_s z - 2 f_s + z\Omega_c + \Omega_c} \\
      &= \frac{z\Omega_c + \Omega_c}{z (2 f_s + \Omega_c) + (\Omega_c - 2 f_s)} \\
      &= \frac{Az + B}{Cz + D}
    \end{align*}
  $$

+ coefficients in powers of $z$:
  $$
    \begin{align*}
      A &= \Omega_c \\
      B &= \Omega_c \\
      C &= 2 f_s + \Omega_c \\
      D &= \Omega_c - 2 f_s
    \end{align*}
  $$

+ convert to standard causal form (powers of $z^{-1}$):
  $$
    \begin{align*}
      H(z) &= \frac{Az + B}{Cz + D} \cdot \frac{z^{-1}}{z^{-1}} \\
           &= \frac{A + B z^{-1}}{C + D z^{-1}} \\
           &= \frac{\frac{A}{C} + \frac{B}{C} z^{-1}}{1 + \frac{D}{C} z^{-1}} \\
           &= \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}}
    \end{align*}
  $$

+ normalize with pre-warping factor $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s}$:
  $$
    \begin{align*}
      b_0 &= \frac{A}{C} = \frac{\Omega_c}{2 f_s + \Omega_c} = \frac{\frac{\Omega_c}{2 f_s}}{1 + \frac{\Omega_c}{2 f_s}} = \frac{K}{1 + K} \\
      b_1 &= \frac{B}{C} = \frac{\Omega_c}{2 f_s + \Omega_c} = b_0 = \frac{K}{1 + K} \\
      a_1 &= \frac{D}{C} = \frac{\Omega_c - 2 f_s}{2 f_s + \Omega_c} = \frac{\frac{\Omega_c}{2 f_s} - 1}{1 + \frac{\Omega_c}{2 f_s}} = \frac{K - 1}{1 + K}
    \end{align*}
  $$

+ difference equation:
  $$
    \begin{align*}
      H(z) = \frac{Y(z)}{X(z)} &= \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}} \\
      Y(z) (1 + a_1 z^{-1}) &= X(z) (b_0 + b_1 z^{-1}) \\
      Y(z) + a_1 z^{-1} Y(z) &= b_0 X(z) + b_1 z^{-1} X(z)
    \end{align*}
  $$
  taking the inverse $\mathcal{Z}$-transform:
  $$
    y[n] + a_1 y[n-1] = b_0 x[n] + b_1 x[n-1]
  $$
  $$
    y[n] = b_0 x[n] + b_1 x[n-1] - a_1 y[n-1]
  $$

+ verification:
  - DC gain ($z = 1$):
    $$
      H(1) = \frac{b_0 + b_1}{1 + a_1} = \frac{2 \frac{K}{1+K}}{1 + \frac{K-1}{1+K}} = \frac{\frac{2K}{1+K}}{\frac{2K}{1+K}} = 1 \quad (0\text{ dB})
    $$
  - Nyquist gain ($z = -1$):
    $$
      H(-1) = \frac{b_0 - b_1}{1 - a_1} = \frac{b_0 - b_0}{1 - a_1} = 0 \quad (-\infty\text{ dB})
    $$

### High-Pass (HPF)

+ analog prototype ($s \larr 1/s$):
  $$
    H(s) = \frac{1}{\frac{1}{s} + 1} = \frac{s}{s + 1}
  $$

+ substitute $s \rarr s/\Omega_c$:
  $$
    H(s) = \frac{s/\Omega_c}{s/\Omega_c + 1} = \frac{s}{s + \Omega_c}
  $$

+ bilinear transform ($s \larr 2 f_s \frac{z - 1}{z + 1}$):
  $$
    \begin{align*}
      H(z) &= \frac{2 f_s \frac{z - 1}{z + 1}}{2 f_s \frac{z - 1}{z + 1} + \Omega_c} \\
           &= \frac{2 f_s (z - 1)}{2 f_s (z - 1) + \Omega_c (z + 1)} \\
           &= \frac{2 f_s z - 2 f_s}{z (2 f_s + \Omega_c) + (\Omega_c - 2 f_s)} \\
           &= \frac{Az + B}{Cz + D}
    \end{align*}
  $$

+ coefficients with $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s}$:
  $$
    \begin{align*}
      A &= 2 f_s \\
      B &= -2 f_s \\
      C &= 2 f_s + \Omega_c \\
      D &= \Omega_c - 2 f_s
    \end{align*}
  $$
  $$
    \begin{align*}
      b_0 &= \frac{A}{C} = \frac{2 f_s}{2 f_s + \Omega_c} = \frac{1}{1 + K} \\
      b_1 &= \frac{B}{C} = \frac{-2 f_s}{2 f_s + \Omega_c} = -\frac{1}{1 + K} = -b_0 \\
      a_1 &= \frac{D}{C} = \frac{\Omega_c - 2 f_s}{2 f_s + \Omega_c} = \frac{K - 1}{1 + K}
    \end{align*}
  $$

+ difference equation:
  $$
    y[n] = b_0 x[n] + b_1 x[n-1] - a_1 y[n-1]
  $$

+ verification:
  - DC gain ($z = 1$):
    $$
      H(1) = \frac{b_0 + b_1}{1 + a_1} = \frac{b_0 - b_0}{1 + a_1} = 0 \quad (-\infty\text{ dB})
    $$
  - Nyquist gain ($z = -1$):
    $$
      H(-1) = \frac{b_0 - b_1}{1 - a_1} = \frac{2 b_0}{1 - a_1} = \frac{\frac{2}{1+K}}{1 - \frac{K-1}{1+K}} = 1 \quad (0\text{ dB})
    $$

### Summary

Let $K = \tan\left(\pi \frac{f_c}{f_s}\right)$:

| Filter | $b_0$ | $b_1$ | $a_1$ | Difference Equation |
| :--- | :---: | :---: | :---: | :--- |
| **Low-Pass (LPF)** | $\frac{K}{1 + K}$ | $\frac{K}{1 + K}$ | $\frac{K - 1}{1 + K}$ | $y[n] = b_0 (x[n] + x[n-1]) - a_1 y[n-1]$ |
| **High-Pass (HPF)** | $\frac{1}{1 + K}$ | $-\frac{1}{1 + K}$ | $\frac{K - 1}{1 + K}$ | $y[n] = b_0 (x[n] - x[n-1]) - a_1 y[n-1]$ |


