# Thermodynamics

## Property packages

`PropertyPackage` composes a component list with an optional equation of
state and an optional activity model. Evaluation rules:

- **Vapor**: ideal-gas enthalpy/entropy plus EOS departure terms.
- **Liquid**: EOS density when attached, else Rackett.
- **K-values**: EOS fugacity ratio, else modified Raoult
  `Kᵢ = γᵢ·P_sat,ᵢ/P`.

## Equations of state

Peng-Robinson and SRK share one generalized cubic implementation with
van der Waals one-fluid mixing and a symmetric kᵢⱼ matrix. Saturation
pressures come from fugacity-equality bisection (valid ~0.3·Tc..Tc).

## Flash

PT flash by successive substitution on K-values with a clamped
Rachford-Rice bisection; PH flash wraps PT in an enthalpy bisection.
Pure-component saturation returns a guarded `degenerate` result.

## The databank

~27 common chemicals with critical constants, acentric factors, and
DIPPR-127 heat capacities — design-estimate grade, fully overridable.
