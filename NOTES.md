# DSP Notes

## 1. First-Order Low-Pass Filter (LPF)

### Analog Prototype
A 1st-order normalized analog Butterworth low-pass filter transfer function is:
$$
H_a(s) = \frac{1}{s + 1}
$$

### Cutoff Angular Frequency & Pre-Warping
- Target digital cutoff frequency: $\omega_c = 2 \pi f_c$
- The bilinear transform maps continuous frequency $\Omega$ to discrete frequency non-linearly. To prevent frequency warping from shifting our cutoff, we calculate the pre-warped analog cutoff:
$$
\Omega_c = 2 f_s \tan\left(\pi \frac{f_c}{f_s}\right)
$$

Let $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s}$.

### Continuous-Time Frequency Scaling ($s \leftarrow s / \Omega_c$)
$$
H_a(s) = \frac{1}{s / \Omega_c + 1} = \frac{\Omega_c}{s + \Omega_c}
$$

### Bilinear Transform
Substitute $s \leftarrow 2 f_s \frac{z - 1}{z + 1}$:
$$
\begin{aligned}
H(z) &= \frac{\Omega_c}{2 f_s \frac{z - 1}{z + 1} + \Omega_c} \\
     &= \frac{\Omega_c (z + 1)}{2 f_s (z - 1) + \Omega_c (z + 1)} \\
     &= \frac{z \Omega_c + \Omega_c}{z (2 f_s + \Omega_c) + (\Omega_c - 2 f_s)}
\end{aligned}
$$

### Standard Causal Form (Powers of $z^{-1}$)
Multiplying numerator and denominator by $z^{-1}$:

$$
  \begin{align*}
    H(z) &= \frac{z \Omega_c + \Omega_c}{z (2 f_s + \Omega_c) + (\Omega_c - 2 f_s)} \cdot \frac{z^{-1}}{z^{-1}} \\
    &= \frac{\Omega_c + \Omega_c z^{-1}}{(2 f_s + \Omega_c) + (\Omega_c - 2 f_s) z^{-1}}
  \end{align*}
$$

and normalizing by coefficients of $z^0$ of **denominator** ($2 f_s + \Omega_c$):

$$
    H(z) = \frac {\frac{\Omega_c}{2f_s + \Omega_c} + \frac{\Omega_c}{2f_s + \Omega_c} z^{-1}}{1+\frac{\Omega_c - 2 f_s}{2f_s + \Omega_c}z^{-1}}
$$

---

$$
H(z) = \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}}
$$

In terms of pre-warping factor $K = \frac{\Omega_c}{2 f_s}$:
$$
\begin{aligned}
b_0 &= \frac{\Omega_c}{2 f_s + \Omega_c} = \frac{K}{1 + K} \\
b_1 &= \frac{\Omega_c}{2 f_s + \Omega_c} = b_0 = \frac{K}{1 + K} \\
a_1 &= \frac{\Omega_c - 2 f_s}{2 f_s + \Omega_c} = \frac{K - 1}{1 + K}
\end{aligned}
$$

### Difference Equation
$$
\frac{Y(z)}{X(z)} = \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}} \implies Y(z)(1 + a_1 z^{-1}) = X(z)(b_0 + b_1 z^{-1})
$$
Taking the inverse $\mathcal{Z}$-transform yields the causal time-domain difference equation:
$$
y[n] = b_0 x[n] + b_1 x[n-1] - a_1 y[n-1]
$$

## 2. First-Order High-Pass Filter (HPF)

### Analog Prototype & Spectral Transformation
Starting from the normalized 1st-order analog low-pass prototype:
$$
H_a(S) = \frac{1}{S + 1}
$$
The standard continuous-time low-pass to high-pass transformation maps:
$$
S \leftarrow \frac{\Omega_c}{s}
$$
Substituting this into the prototype yields the continuous-time transfer function with pre-warped cutoff frequency $\Omega_c$:
$$
H_a(s) = \frac{1}{\frac{\Omega_c}{s} + 1} = \frac{s}{s + \Omega_c}
$$

### Cutoff Angular Frequency & Pre-Warping
As with the LPF, to ensure the digital cutoff frequency precisely matches the target $f_c$ at sampling rate $f_s$:
$$
\Omega_c = 2 f_s \tan\left(\pi \frac{f_c}{f_s}\right)
$$
Let $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s} \implies \Omega_c = 2 f_s K$.

### Bilinear Transform
Substitute $s \leftarrow 2 f_s \frac{z - 1}{z + 1}$:
$$
\begin{aligned}
H(z) &= \frac{2 f_s \frac{z - 1}{z + 1}}{2 f_s \frac{z - 1}{z + 1} + \Omega_c} \\
     &= \frac{2 f_s (z - 1)}{2 f_s (z - 1) + \Omega_c (z + 1)} \\
     &= \frac{2 f_s z - 2 f_s}{z (2 f_s + \Omega_c) + (\Omega_c - 2 f_s)}
\end{aligned}
$$

### Standard Causal Form (Powers of $z^{-1}$)
Multiplying numerator and denominator by $z^{-1}$:
$$
H(z) = \frac{2 f_s - 2 f_s z^{-1}}{(2 f_s + \Omega_c) + (\Omega_c - 2 f_s) z^{-1}}
$$
Normalizing by dividing numerator and denominator by the constant term of the denominator $(2 f_s + \Omega_c)$:
$$
H(z) = \frac{\frac{2 f_s}{2 f_s + \Omega_c} - \frac{2 f_s}{2 f_s + \Omega_c} z^{-1}}{1 + \frac{\Omega_c - 2 f_s}{2 f_s + \Omega_c} z^{-1}}
$$

---

$$
H(z) = \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}}
$$

In terms of pre-warping factor $K = \frac{\Omega_c}{2 f_s}$:
$$
\begin{aligned}
b_0 &= \frac{2 f_s}{2 f_s + \Omega_c} = \frac{1}{1 + K} \\
b_1 &= -\frac{2 f_s}{2 f_s + \Omega_c} = -b_0 = -\frac{1}{1 + K} \\
a_1 &= \frac{\Omega_c - 2 f_s}{2 f_s + \Omega_c} = \frac{K - 1}{1 + K}
\end{aligned}
$$

### Difference Equation
$$
\frac{Y(z)}{X(z)} = \frac{b_0 + b_1 z^{-1}}{1 + a_1 z^{-1}} \implies Y(z)(1 + a_1 z^{-1}) = X(z)(b_0 + b_1 z^{-1})
$$
Taking the inverse $\mathcal{Z}$-transform yields the causal time-domain difference equation:
$$
y[n] = b_0 x[n] + b_1 x[n-1] - a_1 y[n-1]
$$

## 3. Second-Order Low-Pass Filter (LPF)

### Analog Prototype
A general 2nd-order analog low-pass filter prototype is given by:
$$
H_a(S) = \frac{1}{S^2 + \frac{1}{Q} S + 1}
$$
where:
- $Q$ is the quality factor.
- For a **Butterworth** filter (maximally flat magnitude in the passband), the damping ratio is $\zeta = \frac{1}{\sqrt{2}}$, which corresponds to $Q = \frac{1}{2\zeta} = \frac{1}{\sqrt{2}} \approx 0.7071$ and $\frac{1}{Q} = \sqrt{2}$:
$$
H_{a,\text{Butterworth}}(S) = \frac{1}{S^2 + \sqrt{2} S + 1}
$$

### Cutoff Angular Frequency & Pre-Warping
Pre-warping the target digital cutoff frequency $f_c$ at sampling rate $f_s$:
$$
\Omega_c = 2 f_s \tan\left(\pi \frac{f_c}{f_s}\right)
$$
Let $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s} \implies \Omega_c = 2 f_s K$.

### Continuous-Time Frequency Scaling ($S \leftarrow s / \Omega_c$)
$$
H_a(s) = \frac{1}{\left(\frac{s}{\Omega_c}\right)^2 + \frac{1}{Q}\left(\frac{s}{\Omega_c}\right) + 1} = \frac{\Omega_c^2}{s^2 + \frac{\Omega_c}{Q} s + \Omega_c^2}
$$

### Bilinear Transform
Substitute $s \leftarrow 2 f_s \frac{z - 1}{z + 1}$:
$$
\begin{aligned}
H(z) &= \frac{\Omega_c^2}{\left(2 f_s \frac{z-1}{z+1}\right)^2 + \frac{\Omega_c}{Q}\left(2 f_s \frac{z-1}{z+1}\right) + \Omega_c^2} \\
     &= \frac{\Omega_c^2 (z+1)^2}{(2 f_s)^2 (z-1)^2 + \frac{\Omega_c}{Q} (2 f_s)(z-1)(z+1) + \Omega_c^2 (z+1)^2}
\end{aligned}
$$
Dividing numerator and denominator by $(2 f_s)^2$ and substituting $K = \frac{\Omega_c}{2 f_s}$:
$$
H(z) = \frac{K^2 (z+1)^2}{(z-1)^2 + \frac{K}{Q} (z^2 - 1) + K^2 (z+1)^2}
$$
Expanding the polynomials:
- **Numerator**:
$$
K^2 (z^2 + 2z + 1) = K^2 z^2 + 2 K^2 z + K^2
$$
- **Denominator**:
$$
\begin{aligned}
&(z^2 - 2z + 1) + \frac{K}{Q}(z^2 - 1) + K^2 (z^2 + 2z + 1) \\
&= \left(1 + \frac{K}{Q} + K^2\right) z^2 + 2(K^2 - 1) z + \left(1 - \frac{K}{Q} + K^2\right)
\end{aligned}
$$

### Standard Causal Form (Powers of $z^{-1}$)
Multiplying numerator and denominator by $z^{-2}$:
$$
H(z) = \frac{K^2 + 2 K^2 z^{-1} + K^2 z^{-2}}{\left(1 + \frac{K}{Q} + K^2\right) + 2(K^2 - 1) z^{-1} + \left(1 - \frac{K}{Q} + K^2\right) z^{-2}}
$$
Define the normalization factor $a_0$:
$$
a_0 = 1 + \frac{K}{Q} + K^2
$$
Dividing numerator and denominator by $a_0$:
$$
H(z) = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{1 + a_1 z^{-1} + a_2 z^{-2}}
$$
where:
$$
\begin{aligned}
b_0 &= \frac{K^2}{a_0} \\
b_1 &= \frac{2 K^2}{a_0} = 2 b_0 \\
b_2 &= \frac{K^2}{a_0} = b_0 \\
a_1 &= \frac{2(K^2 - 1)}{a_0} \\
a_2 &= \frac{1 - \frac{K}{Q} + K^2}{a_0}
\end{aligned}
$$

For a Butterworth filter ($Q = \frac{1}{\sqrt{2}}$):
$$
\begin{aligned}
a_0 &= 1 + \sqrt{2} K + K^2 \\
a_1 &= \frac{2(K^2 - 1)}{a_0} \\
a_2 &= \frac{1 - \sqrt{2} K + K^2}{a_0}
\end{aligned}
$$

### Difference Equation
$$
\frac{Y(z)}{X(z)} = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{1 + a_1 z^{-1} + a_2 z^{-2}} \implies Y(z)(1 + a_1 z^{-1} + a_2 z^{-2}) = X(z)(b_0 + b_1 z^{-1} + b_2 z^{-2})
$$
Taking the inverse $\mathcal{Z}$-transform yields the 2nd-order difference equation (biquad Direct Form I):
$$
y[n] = b_0 x[n] + b_1 x[n-1] + b_2 x[n-2] - a_1 y[n-1] - a_2 y[n-2]
$$

## 4. Second-Order High-Pass Filter (HPF)

### Analog Prototype & Spectral Transformation
Starting from the normalized 2nd-order analog low-pass prototype:
$$
H_{LP}(S) = \frac{1}{S^2 + \frac{1}{Q} S + 1}
$$
Applying the low-pass to high-pass spectral transformation $S \leftarrow \frac{\Omega_c}{s}$:
$$
\begin{aligned}
H_a(s) &= \frac{1}{\left(\frac{\Omega_c}{s}\right)^2 + \frac{1}{Q}\left(\frac{\Omega_c}{s}\right) + 1} \\
       &= \frac{s^2}{s^2 + \frac{\Omega_c}{Q} s + \Omega_c^2}
\end{aligned}
$$
For a Butterworth filter ($Q = \frac{1}{\sqrt{2}}$):
$$
H_{a,\text{Butterworth}}(s) = \frac{s^2}{s^2 + \sqrt{2} \Omega_c s + \Omega_c^2}
$$

### Cutoff Angular Frequency & Pre-Warping
As before, pre-warping cutoff $f_c$ at sampling rate $f_s$:
$$
\Omega_c = 2 f_s \tan\left(\pi \frac{f_c}{f_s}\right)
$$
Let $K = \tan\left(\pi \frac{f_c}{f_s}\right) = \frac{\Omega_c}{2 f_s} \implies \Omega_c = 2 f_s K$.

### Bilinear Transform
Substitute $s \leftarrow 2 f_s \frac{z - 1}{z + 1}$:
$$
\begin{aligned}
H(z) &= \frac{\left(2 f_s \frac{z-1}{z+1}\right)^2}{\left(2 f_s \frac{z-1}{z+1}\right)^2 + \frac{\Omega_c}{Q}\left(2 f_s \frac{z-1}{z+1}\right) + \Omega_c^2} \\
     &= \frac{(2 f_s)^2 (z-1)^2}{(2 f_s)^2 (z-1)^2 + \frac{\Omega_c}{Q} (2 f_s)(z-1)(z+1) + \Omega_c^2 (z+1)^2}
\end{aligned}
$$
Dividing numerator and denominator by $(2 f_s)^2$ and substituting $K = \frac{\Omega_c}{2 f_s}$:
$$
H(z) = \frac{(z-1)^2}{(z-1)^2 + \frac{K}{Q} (z^2 - 1) + K^2 (z+1)^2}
$$
Expanding the polynomials:
- **Numerator**:
$$
(z-1)^2 = z^2 - 2z + 1
$$
- **Denominator** (identical to 2nd-order LPF):
$$
\left(1 + \frac{K}{Q} + K^2\right) z^2 + 2(K^2 - 1) z + \left(1 - \frac{K}{Q} + K^2\right)
$$

### Standard Causal Form (Powers of $z^{-1}$)
Multiplying numerator and denominator by $z^{-2}$:
$$
H(z) = \frac{1 - 2 z^{-1} + z^{-2}}{\left(1 + \frac{K}{Q} + K^2\right) + 2(K^2 - 1) z^{-1} + \left(1 - \frac{K}{Q} + K^2\right) z^{-2}}
$$
Define the normalization factor $a_0$:
$$
a_0 = 1 + \frac{K}{Q} + K^2
$$
Dividing numerator and denominator by $a_0$:
$$
H(z) = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{1 + a_1 z^{-1} + a_2 z^{-2}}
$$
where:
$$
\begin{aligned}
b_0 &= \frac{1}{a_0} \\
b_1 &= -\frac{2}{a_0} = -2 b_0 \\
b_2 &= \frac{1}{a_0} = b_0 \\
a_1 &= \frac{2(K^2 - 1)}{a_0} \\
a_2 &= \frac{1 - \frac{K}{Q} + K^2}{a_0}
\end{aligned}
$$

For a Butterworth filter ($Q = \frac{1}{\sqrt{2}}$):
$$
\begin{aligned}
a_0 &= 1 + \sqrt{2} K + K^2 \\
a_1 &= \frac{2(K^2 - 1)}{a_0} \\
a_2 &= \frac{1 - \sqrt{2} K + K^2}{a_0}
\end{aligned}
$$

### Difference Equation
$$
\frac{Y(z)}{X(z)} = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{1 + a_1 z^{-1} + a_2 z^{-2}} \implies Y(z)(1 + a_1 z^{-1} + a_2 z^{-2}) = X(z)(b_0 + b_1 z^{-1} + b_2 z^{-2})
$$
Taking the inverse $\mathcal{Z}$-transform yields:
$$
y[n] = b_0 x[n] + b_1 x[n-1] + b_2 x[n-2] - a_1 y[n-1] - a_2 y[n-2]
$$

## 5. Summary of Filter Coefficients

With $K = \tan\left(\pi \frac{f_c}{f_s}\right)$:

| Filter Type | Order | Normalization $a_0$ | $b_0$ | $b_1$ | $b_2$ | $a_1$ | $a_2$ |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **LPF** | 1st | $1 + K$ | $\frac{K}{a_0}$ | $\frac{K}{a_0}$ | — | $\frac{K - 1}{a_0}$ | — |
| **HPF** | 1st | $1 + K$ | $\frac{1}{a_0}$ | $-\frac{1}{a_0}$ | — | $\frac{K - 1}{a_0}$ | — |
| **LPF** | 2nd | $1 + \frac{K}{Q} + K^2$ | $\frac{K^2}{a_0}$ | $\frac{2 K^2}{a_0}$ | $\frac{K^2}{a_0}$ | $\frac{2(K^2 - 1)}{a_0}$ | $\frac{1 - \frac{K}{Q} + K^2}{a_0}$ |
| **HPF** | 2nd | $1 + \frac{K}{Q} + K^2$ | $\frac{1}{a_0}$ | $-\frac{2}{a_0}$ | $\frac{1}{a_0}$ | $\frac{2(K^2 - 1)}{a_0}$ | $\frac{1 - \frac{K}{Q} + K^2}{a_0}$ |

*Note: For 2nd-order Butterworth responses, set $Q = \frac{1}{\sqrt{2}} \implies \frac{1}{Q} = \sqrt{2}$.*