"""A small two-pass 6502 assembler.

Enough of one to build the demo ROM: labels (global and `@local`), constants,
the usual directives, and expressions with the `<`/`>` byte-select prefixes.
Zero-page operands are chosen automatically when the value is already known,
so zero-page variables must be defined before they are used.
"""

import re

# mnemonic -> {addressing mode: opcode}
OPCODES = {}


def _build_table():
    rows = """
    BRK imp 00 | ORA izx 01 | ORA zp  05 | ASL zp  06 | PHP imp 08 | ORA imm 09
    ASL acc 0A | ORA abs 0D | ASL abs 0E | BPL rel 10 | ORA izy 11 | ORA zpx 15
    ASL zpx 16 | CLC imp 18 | ORA aby 19 | ORA abx 1D | ASL abx 1E | JSR abs 20
    AND izx 21 | BIT zp  24 | AND zp  25 | ROL zp  26 | PLP imp 28 | AND imm 29
    ROL acc 2A | BIT abs 2C | AND abs 2D | ROL abs 2E | BMI rel 30 | AND izy 31
    AND zpx 35 | ROL zpx 36 | SEC imp 38 | AND aby 39 | AND abx 3D | ROL abx 3E
    RTI imp 40 | EOR izx 41 | EOR zp  45 | LSR zp  46 | PHA imp 48 | EOR imm 49
    LSR acc 4A | JMP abs 4C | EOR abs 4D | LSR abs 4E | BVC rel 50 | EOR izy 51
    EOR zpx 55 | LSR zpx 56 | CLI imp 58 | EOR aby 59 | EOR abx 5D | LSR abx 5E
    RTS imp 60 | ADC izx 61 | ADC zp  65 | ROR zp  66 | PLA imp 68 | ADC imm 69
    ROR acc 6A | JMP ind 6C | ADC abs 6D | ROR abs 6E | BVS rel 70 | ADC izy 71
    ADC zpx 75 | ROR zpx 76 | SEI imp 78 | ADC aby 79 | ADC abx 7D | ROR abx 7E
    STA izx 81 | STY zp  84 | STA zp  85 | STX zp  86 | DEY imp 88 | TXA imp 8A
    STY abs 8C | STA abs 8D | STX abs 8E | BCC rel 90 | STA izy 91 | STY zpx 94
    STA zpx 95 | STX zpy 96 | TYA imp 98 | STA aby 99 | TXS imp 9A | STA abx 9D
    LDY imm A0 | LDA izx A1 | LDX imm A2 | LDY zp  A4 | LDA zp  A5 | LDX zp  A6
    TAY imp A8 | LDA imm A9 | TAX imp AA | LDY abs AC | LDA abs AD | LDX abs AE
    BCS rel B0 | LDA izy B1 | LDY zpx B4 | LDA zpx B5 | LDX zpy B6 | CLV imp B8
    LDA aby B9 | TSX imp BA | LDY abx BC | LDA abx BD | LDX aby BE | CPY imm C0
    CMP izx C1 | CPY zp  C4 | CMP zp  C5 | DEC zp  C6 | INY imp C8 | CMP imm C9
    DEX imp CA | CPY abs CC | CMP abs CD | DEC abs CE | BNE rel D0 | CMP izy D1
    CMP zpx D5 | DEC zpx D6 | CLD imp D8 | CMP aby D9 | CMP abx DD | DEC abx DE
    CPX imm E0 | SBC izx E1 | CPX zp  E4 | SBC zp  E5 | INC zp  E6 | INX imp E8
    SBC imm E9 | NOP imp EA | CPX abs EC | SBC abs ED | INC abs EE | BEQ rel F0
    SBC izy F1 | SBC zpx F5 | INC zpx F6 | SED imp F8 | SBC aby F9 | SBC abx FD
    INC abx FE
    """
    tokens = rows.replace("|", " ").split()
    for index in range(0, len(tokens), 3):
        mnemonic, mode, code = tokens[index:index + 3]
        OPCODES.setdefault(mnemonic, {})[mode] = int(code, 16)


_build_table()

MODE_SIZES = {
    "imp": 1, "acc": 1, "imm": 2, "zp": 2, "zpx": 2, "zpy": 2, "rel": 2,
    "izx": 2, "izy": 2, "abs": 3, "abx": 3, "aby": 3, "ind": 3,
}


class AsmError(Exception):
    pass


class Assembler:
    def __init__(self):
        self.symbols = {}
        self.output = {}      # address -> byte
        self.pc = 0x8000
        self.scope = ""       # current global label, for @local names
        self.strict = False   # raise on unknown symbols (final pass only)

    # ------------------------------------------------------------ expressions

    def resolve(self, name):
        if name.startswith("@"):
            name = self.scope + name
        if name in self.symbols:
            return self.symbols[name]
        if self.strict:
            raise AsmError("undefined symbol: %s" % name)
        return 0xFFFF  # assume worst case so sizes stay stable across passes

    def evaluate(self, text):
        text = text.strip()
        if not text:
            raise AsmError("empty expression")

        # $hex, %binary, 'c' character literals
        def number(match):
            token = match.group(0)
            if token.startswith("$"):
                return str(int(token[1:], 16))
            if token.startswith("%"):
                return str(int(token[1:], 2))
            return token

        text = re.sub(r"\$[0-9A-Fa-f]+|%[01]+", number, text)
        text = re.sub(r"'(.)'", lambda m: str(ord(m.group(1))), text)

        # Symbols become dictionary lookups so Python's parser can do the rest.
        def symbol(match):
            name = match.group(0)
            if re.fullmatch(r"\d+", name):
                return name
            return "S(%r)" % name

        text = re.sub(r"@?[A-Za-z_][A-Za-z0-9_]*", symbol, text)
        text = text.replace("*", "S('*')") if text.strip() == "*" else text

        try:
            return int(eval(text, {"__builtins__": {}}, {"S": self.resolve}))
        except AsmError:
            raise
        except Exception as exc:
            raise AsmError("bad expression %r: %s" % (text, exc))

    def byte_select(self, text):
        """Handle the `<expr` / `>expr` low/high byte prefixes."""
        text = text.strip()
        if text.startswith("<"):
            return self.evaluate(text[1:]) & 0xFF
        if text.startswith(">"):
            return (self.evaluate(text[1:]) >> 8) & 0xFF
        return self.evaluate(text)

    # -------------------------------------------------------------- emission

    def emit(self, *values):
        for value in values:
            self.output[self.pc] = value & 0xFF
            self.pc += 1

    # ------------------------------------------------------------ directives

    def directive(self, name, operand):
        if name in (".org",):
            self.pc = self.evaluate(operand)
        elif name in (".byte", ".db"):
            for item in split_commas(operand):
                item = item.strip()
                if item.startswith('"'):
                    for char in unquote(item):
                        self.emit(ord(char))
                else:
                    self.emit(self.byte_select(item))
        elif name in (".word", ".dw"):
            for item in split_commas(operand):
                value = self.evaluate(item)
                self.emit(value & 0xFF, value >> 8)
        elif name in (".res", ".ds"):
            parts = split_commas(operand)
            count = self.evaluate(parts[0])
            fill = self.evaluate(parts[1]) if len(parts) > 1 else 0
            for _ in range(count):
                self.emit(fill)
        elif name == ".align":
            boundary = self.evaluate(operand)
            while self.pc % boundary:
                self.emit(0)
        else:
            raise AsmError("unknown directive %s" % name)

    # ----------------------------------------------------------- instructions

    def instruction(self, mnemonic, operand, size_only=False):
        modes = OPCODES[mnemonic]
        operand = operand.strip()

        if not operand or operand.upper() == "A":
            mode, value = ("acc" if "acc" in modes else "imp"), None
        elif operand.startswith("#"):
            mode, value = "imm", self.byte_select(operand[1:])
        elif re.fullmatch(r"\(.*\)\s*,\s*[Yy]", operand):
            mode = "izy"
            value = self.evaluate(operand[1:operand.rindex(")")])
        elif re.fullmatch(r"\(.*,\s*[Xx]\)", operand):
            inner = operand[1:operand.rindex(")")]
            mode = "izx"
            value = self.evaluate(inner[:inner.rindex(",")])
        elif operand.startswith("(") and operand.endswith(")"):
            mode, value = "ind", self.evaluate(operand[1:-1])
        elif re.search(r",\s*[Xx]$", operand):
            value = self.evaluate(operand[:operand.rindex(",")])
            mode = "zpx" if value < 0x100 and "zpx" in modes else "abx"
        elif re.search(r",\s*[Yy]$", operand):
            value = self.evaluate(operand[:operand.rindex(",")])
            mode = "zpy" if value < 0x100 and "zpy" in modes else "aby"
        elif "rel" in modes:
            mode, value = "rel", self.evaluate(operand)
        else:
            value = self.evaluate(operand)
            mode = "zp" if value < 0x100 and "zp" in modes else "abs"

        if mode not in modes:
            raise AsmError("%s does not support %s addressing" % (mnemonic, mode))

        size = MODE_SIZES[mode]
        if size_only:
            self.pc += size
            return

        self.emit(modes[mode])
        if mode == "rel":
            offset = value - (self.pc + 1)
            if not -128 <= offset <= 127:
                # Forward references are still placeholders on early passes.
                if self.strict:
                    raise AsmError(
                        "branch out of range at $%04X (%d bytes)" % (self.pc - 1, offset)
                    )
                offset = 0
            self.emit(offset & 0xFF)
        elif size == 2:
            self.emit(value)
        elif size == 3:
            self.emit(value & 0xFF, value >> 8)

    # ---------------------------------------------------------------- passes

    def pass_over(self, lines, define_symbols):
        self.pc = 0x8000
        self.scope = ""
        self.output = {}

        for number, raw in lines:
            line = raw.split(";")[0].rstrip()
            if not line.strip():
                continue

            try:
                # Labels start in column 0.
                match = re.match(r"^(@?[A-Za-z_][A-Za-z0-9_]*):\s*", line)
                if match and not line.startswith((" ", "\t")):
                    name = match.group(1)
                    if name.startswith("@"):
                        name = self.scope + name
                    else:
                        self.scope = name
                    if define_symbols:
                        self.symbols[name] = self.pc
                    line = line[match.end():]
                    if not line.strip():
                        continue

                # Constant definition: name = expression
                match = re.match(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+)$", line)
                if match:
                    self.symbols[match.group(1)] = self.evaluate(match.group(2))
                    continue

                parts = line.strip().split(None, 1)
                head = parts[0]
                operand = parts[1] if len(parts) > 1 else ""

                if head.startswith("."):
                    self.directive(head.lower(), operand)
                elif head.upper() in OPCODES:
                    self.instruction(head.upper(), operand, size_only=not define_symbols)
                else:
                    raise AsmError("unknown instruction %r" % head)
            except AsmError as exc:
                raise AsmError("line %d: %s\n    %s" % (number, exc, raw.strip()))

    def assemble(self, source):
        lines = list(enumerate(source.splitlines(), 1))
        # Pass 1 measures sizes and records labels; later passes settle any
        # forward references that changed an operand's width.
        for _ in range(3):
            self.pass_over(lines, define_symbols=True)
        self.strict = True
        self.pass_over(lines, define_symbols=True)
        return self.output


def split_commas(text):
    """Split on commas that are not inside a quoted string."""
    parts, current, in_string = [], "", False
    for char in text:
        if char == '"':
            in_string = not in_string
        if char == "," and not in_string:
            parts.append(current)
            current = ""
        else:
            current += char
    if current.strip():
        parts.append(current)
    return parts


def unquote(text):
    return text.strip()[1:-1]


def assemble(source):
    return Assembler().assemble(source)
