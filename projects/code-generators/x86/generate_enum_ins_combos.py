#!/usr/bin/python3
import re
from pathlib import Path

# Function to read enum text from a file
def read_enum_from_file(file_path):
    return file_path.read_text()

# Get the path of the current script and the code.rs file
script_path = Path(__file__).resolve()
code_rs_path = script_path.parent / 'code.rs'

# Read the enum text from code.rs
enum_text = read_enum_from_file(code_rs_path)

# Regular expression to match lines with specified operands inside backticks and extract the content
pattern = re.compile(r"^\s*\/\/\/\s*`.*?(r\/m8|r\/m16|r\/m32|r\/m64|xmm\d+\/m128|ymm\d+\/m256|zmm\d+\/m512|r8, m|r16, m|r32, m|r64, m).*?`", re.MULTILINE)

# Dictionary to store unique items grouped by number of commas
grouped_items = {}

for line_i, line in enumerate(enum_text.splitlines()):
    if pattern.search(line):
        content_inside_backticks = re.search(r"`([^`]+)`", line).group(1)
        words_after_first = ' '.join(content_inside_backticks.split()[1:]) # Splitting the content to remove the first word
        
        # Normalizing XMM registers
        words_after_first = re.sub(r"(zmm|xmm|ymm)(\d+)", r"xmm\2", words_after_first)

        # Count the number of operands
        num_operands = words_after_first.count(',') + 1

        # Add to the appropriate group
        if num_operands not in grouped_items:
            grouped_items[num_operands] = set()

        grouped_items[num_operands].add(words_after_first)

# Print sorted groups
for num_operands in sorted(grouped_items.keys()):
    print(f"{num_operands} operands:")
    for item in sorted(grouped_items[num_operands]):
        print(item)
    print() # Print a newline for better readability between groups