import random
import hashlib
from keys import PoseidonKeys

P = 2**251 + 17*2**192 + 1

class PoseidonStwoLike:
    """
    An implementation of Poseidon matching the Stwo implementation, using the same keys.
    """
    def __init__(self, state):
        self.state = state

    def _full_round(self, keys):
        x, y, z = [a**3 for a in self.state]
        state = [3 * x + y + z, x - y + z, x + y - 2 * z]
        self.state = [(a + k)%P for a, k in zip(state, keys)]

    def _partial_round(self, half_key):
        z0_3, z1, z1_3, z2 = self.state
        z2_3 = (z2**3) % P
        z3 = (2*(4*z0_3 + 2*z1 + 3*z1_3 + z2 - z2_3 + half_key)) % P
        self.state = (z1_3, z2, z2_3, z3)

    def hash(self):
        """
        Applies the Hades permutation to the state and returns the new state.
        """
        self.state = [(a+k)%P for a,k in zip(self.state, PoseidonKeys.entry_keys)]
        for ks in PoseidonKeys.full_keys_0_4:
            self._full_round(ks)
        x4, y4, z4 = self.state
        z4_3 = (z4**3) % P
        z5 = (x4 + y4 - 2*z4_3 + PoseidonKeys.z5_key) % P
        z5_3 = (z5**3) % P
        z6 = (4*x4 + 2*z4_3 - 2*z5_3 + PoseidonKeys.z6_key) % P
        self.state = (z4_3, z5, z5_3, z6)
        for k in PoseidonKeys.partial_round_half_keys:
            self._partial_round(k)
        z85_3, z86, z86_3, z87 = self.state
        y87 = (2*z86 + 4*z85_3 + z86_3 + PoseidonKeys.y87_key) % P
        x87 = (y87 + 2*z87 + 4*z86_3 + PoseidonKeys.x87_key) % P
        self.state = [x87, y87, z87]
        for ks in PoseidonKeys.full_keys_87_91:
            self._full_round(ks)
        return self.state

class Poseidon:
    """
    A standard implementation of Poseidon, using the standard round keys.
    """
    def __init__(self, state):
        self.state = state

    def _round(self, keys, is_full=True):
        pows = [3, 3, 3] if is_full else [1, 1, 3]
        x, y, z = [((s + k)**e) % P for s, k, e in zip(self.state, keys, pows)]
        x, y, z = 3*x + y + z, x - y + z, x + y - 2*z
        self.state = [s % P for s in [x, y, z]]

    def hash(self):
        """
        Applies the Hades permutation to the state and returns the new state.
        """
        fulls = [True]*4 + [False]*83 + [True]*4
        for keys, is_full in zip(PoseidonKeys.RoundKeys, fulls):
            self._round(keys, is_full)
        return self.state

    def full_trace(self):
        """
        Computes the full Hades permutation, and returns the trace of all intermediary states, after
        the add-key step as well as after the cubing step. This trace can then be used to extract
        the keys that should be added in the linear combinations used in the Stwo implementation.
        """
        fulls = [True]*4 + [False]*83 + [True]*4
        states_with_keys = []
        states_with_keys_cubed = []
        for keys, is_full in zip(PoseidonKeys.RoundKeys, fulls):
            state_with_keys = [(a + key) % P for a, key in zip(self.state, keys)]
            states_with_keys.append(state_with_keys)
            state_with_keys_cubed = [(a**3)  % P for a in state_with_keys]
            states_with_keys_cubed.append(state_with_keys_cubed)
            self._round(keys, is_full)
        states_with_keys.append(self.state)
        return states_with_keys, states_with_keys_cubed

def generate_standard_round_keys() -> int:
    """
    Generates the rounds keys for the Hades permutation using sha256 values modulo P.
    """
    return [
        [
            int(hashlib.sha256(f'Hades{idx}'.encode('utf-8')).hexdigest(), 16) % P
            for idx in range(3 * i, 3 * i + 3)
        ] for i in range(91)
    ]

def monomial_to_coef(mon: str):
    """
    Parses a monominal from the linear combination into indices in the trace array.
    """
    if '*' in mon:
        coef_str, trace_elt_str = mon.split('*')
        coef = int(coef_str)
    elif mon.startswith('-'):
        coef, trace_elt_str = -1, mon[1:]
    else:
        coef, trace_elt_str = 1, mon
    state_element = {'x': 0, 'y': 1, 'z': 2}[trace_elt_str[0]]
    is_cubed = trace_elt_str.endswith('^3')
    round_num = int(trace_elt_str[1:-2] if is_cubed else trace_elt_str[1:])
    return (coef, (state_element, round_num, is_cubed))

def trace_lin_comb(trace, combination: str):
    """
    Computes a linear combination of the trace elements.

    Trace elements are given in the form [xyz]{round_number} for plain states, or ending with '^3'
    for their cubes. They can appear with a coefficient in the form {coef}*{trace_elt} or without
    (for coefficient 1). They are separated by ' + ' or ' - ' (for negative coefficients).
    """
    combination = combination.replace(' - ', ' + -')
    coefs = map(monomial_to_coef, combination.split(' + '))
    return sum([coef * trace[is_cubed][round_num][state_element] for
             coef, (state_element, round_num, is_cubed) in coefs]) % P

def generate_keys_for_stwo_poseidon(state = [0, 0, 0]):
    """
    Generates the keys the stwo-like Poseidon implementation from linear combination of the trace
    of a standard Poseidon execution.
    """
    trace = Poseidon(state).full_trace()
    entry_keys = PoseidonKeys.RoundKeys[0]
    full_keys_0_4 = PoseidonKeys.RoundKeys[1:5]
    full_keys_87_91 = PoseidonKeys.RoundKeys[88:91] + [[0, 0, 0]]
    transition_keys = {
        name: trace_lin_comb(trace, combination) for name, combination in [
            ('z5', 'z5 - x4 - y4 + 2*z4^3'),
            ('z6', 'z6 - 4*x4 - 2*z4^3 + 2*z5^3'),
            ('y87', 'y87 - 4*z85^3 - 2*z86 - z86^3'),
            ('x87', 'x87 - 4*z86^3 - y87 - 2*z87'),
        ]
    }
    partial_round_half_keys = [
        trace_lin_comb(
            trace, f'{(P + 1) // 2}*z{i+3} - 4*z{i}^3 - 2*z{i+1} - 3*z{i+1}^3 - z{i+2} + z{i+2}^3'
        ) for i in range(4, 85)
    ]
    return entry_keys, full_keys_0_4, transition_keys, partial_round_half_keys, full_keys_87_91



#####################                      Tests                          ##########################

def test_poseidon_golden():
    assert Poseidon([0,0,0]).hash() == [
        3446325744004048536138401612021367625846492093718951375866996507163446763827,
        1590252087433376791875644726012779423683501236913937337746052470473806035332,
        867921192302518434283879514999422690776342565400001269945778456016268852423
    ]

def test_generate_keys():
    assert PoseidonKeys.RoundKeys == generate_standard_round_keys()

    all_keys = generate_keys_for_stwo_poseidon()
    assert all_keys == (
        PoseidonKeys.entry_keys,
        PoseidonKeys.full_keys_0_4,
        {
            'z5': PoseidonKeys.z5_key,
            'z6': PoseidonKeys.z6_key,
            'y87': PoseidonKeys.y87_key,
            'x87': PoseidonKeys.x87_key,
        },
        PoseidonKeys.partial_round_half_keys,
        PoseidonKeys.full_keys_87_91,
    )

def test_compare_poseidon_impls():
    test_states = [[0, 0, 0], [1, 2, 3], [random.randint(0, P-1) for _ in range(3)]]
    for state in test_states:
        assert Poseidon(state).hash() == PoseidonStwoLike(state).hash()

test_poseidon_golden()
test_generate_keys()
test_compare_poseidon_impls()
