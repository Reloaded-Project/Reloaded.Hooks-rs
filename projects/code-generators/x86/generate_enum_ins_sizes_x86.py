#!/usr/bin/python3
import re
from pathlib import Path

# Function to read enum text from a file
def read_enum_from_file(file_path):
    return file_path.read_text()

# Function to calculate byte size based on the opcode
def calculate_byte_size(opcode):
    segments = opcode.split()
    base_size = len(segments)

    # Check for specific prefixes and adjust base size
    # This is probably very, very wrong, as my understanding of x86 encoding is
    # very limited.
    for segment in segments:
        if segment in ['o32', 'NP']:
            base_size -= 1 # 32-bit is native instruction size, NP is no prefix
        if segment in ['iw']:
            base_size += 1 # 2 byte immediate
        if segment in ['id']:
            base_size += 3 # 4 byte immediate
        if segment.startswith('VEX'):
            base_size += 1 # VEX Prefix (2 byte)
        if segment.startswith('EVEX'):
            base_size += 3 # EVEX Prefix (4 bytes)

    return base_size

# Get the path of the current script and the code.rs file
script_path = Path(__file__).resolve()
code_rs_path = script_path.parent / 'code.rs'

# Read the enum text from code.rs
enum_text = read_enum_from_file(code_rs_path)

# Regular expression to match the instruction name on the first line containing specified patterns
# Pattern breakdown:
# ^\s*\/\/\/\s*` : Matches the beginning of a comment line.
# .*?(r\/m8|r\/m16|r\/m32|r\/m64|xmm\d+\/m128|ymm\d+\/m256|zmm\d+\/m512|r8, m|r16, m|r32, m|r64, m) : Non-greedy match of instruction name until specific memory operands.
# .*?` : Continues matching any characters non-greedily until end of instruction name.
# \n.*\n.*?` : Skips the next two lines and finds the start of opcode.
# ([^`]+) : Captures the opcode inside backticks.
# (?:.*\n){5} : Skips the next five lines.
# .*?(\w+)\s+=\s+\d+ : Captures the enum variant name followed by an equals sign and a number.
pattern = re.compile(r"^\s*\/\/\/\s*`.*?(r\/m8|r\/m16|r\/m32|r\/m64|xmm\d+\/m128|ymm\d+\/m256|zmm\d+\/m512|r8, m|r16, m|r32, m|r64, m).*?`\n.*\n.*?`([^`]+)`(?:.*\n){5}.*?(\w+)\s+=\s+\d+", re.MULTILINE)
matches = pattern.findall(enum_text)

# Calculate byte size for each instruction and create match arms
match_arms = []
for description, opcode, name in matches:
    byte_size = calculate_byte_size(opcode)
    match_arms.append(f"    Code::{name} => {byte_size},")

# Joining the match arms into a single string
match_code = "match code {\n" + "\n".join(match_arms) + "\n}"

print(match_code)