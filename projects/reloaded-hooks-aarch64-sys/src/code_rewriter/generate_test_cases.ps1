# Set the file paths relative to the script location
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$instructionsPath = Join-Path $scriptPath "sample_instructions.txt"
$hexNumbersPath = Join-Path $scriptPath "sample_instructions_bytes.txt"

# Check if input files exist
if (-not (Test-Path $instructionsPath) -or -not (Test-Path $hexNumbersPath)) {
    Write-Error "Input files do not exist."
    exit
}

# Read both files
$instructions = Get-Content $instructionsPath
$hexNumbers = Get-Content $hexNumbersPath

for ($i = 0; $i -lt $instructions.Length; $i++) {
    # Extract the instruction and corresponding hex number
    $instructionLine = $instructions[$i].ToLower().Trim()
    $hexNumber = $hexNumbers[$i].Trim()

    # Remove spaces from hex number if present
    $hexNumber = $hexNumber.Replace(" ", "")

    # Format the hex number into the required Rust format
    $hexFormatted = "0x$hexNumber`_u32.to_be()"

    # Extract the instruction mnemonic (name) by taking the first word of the instruction
    $instructionName = ($instructionLine -split '\s')[0].Trim()

    # Construct the test case line
    $testCaseLine = "#[case::${instructionName}($hexFormatted, InsType::Unknown)] // $instructionLine"

    # Print to the standard output
    Write-Output $testCaseLine
}
