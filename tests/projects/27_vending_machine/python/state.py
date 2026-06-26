# State machine vocabulary for the vending machine.
# States and events are modelled as integer-tagged enums so the control
# flow mirrors the Iron `enum` / `match` version exactly.

# --- States -----------------------------------------------------------
ST_IDLE = 0
ST_COLLECTING = 1
ST_DISPENSING = 2
ST_REFUNDING = 3


def state_name(s: int) -> str:
    if s == ST_IDLE:
        return "IDLE"
    if s == ST_COLLECTING:
        return "COLLECTING"
    if s == ST_DISPENSING:
        return "DISPENSING"
    if s == ST_REFUNDING:
        return "REFUNDING"
    return "UNKNOWN"


# --- Events -----------------------------------------------------------
EV_INSERT = 0
EV_SELECT = 1
EV_CANCEL = 2
EV_TICK = 3


def event_name(e: int) -> str:
    if e == EV_INSERT:
        return "INSERT"
    if e == EV_SELECT:
        return "SELECT"
    if e == EV_CANCEL:
        return "CANCEL"
    if e == EV_TICK:
        return "TICK"
    return "NONE"


# --- Coin denominations (cents), largest first ------------------------
def coins() -> list:
    return [200, 100, 50, 25, 10, 5]


def is_valid_coin(c: int) -> int:
    for d in coins():
        if d == c:
            return 1
    return 0


# Greedy change: return the list of coin values that sum to `amount`,
# largest denomination first. Returns an empty list when amount is 0.
def make_change(amount: int) -> list:
    out = []
    remaining = amount
    for d in coins():
        while remaining >= d:
            out.append(d)
            remaining = remaining - d
    return out


# Number of coins needed to render `amount` (length of make_change).
def change_count(amount: int) -> int:
    return len(make_change(amount))
