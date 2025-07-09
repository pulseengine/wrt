#!/bin/bash

# Fix remaining unterminated string patterns in async files

find wrt-component/src/async_/ -name "*.rs" | while read -r file; do
    echo "Fixing remaining strings in $file"
    
    # Fix pattern: "Missing error messageMissing messageMissing message Error message" -> "Missing error message"
    perl -i -pe 's/"Missing error messageMissing messageMissing message Error message"/"Missing error message"/g' "$file"
    
    # Fix pattern: "Missing error message Error message" -> "Missing error message"
    perl -i -pe 's/"Missing error message Error message"/"Missing error message"/g' "$file"
    
    # Fix pattern: "Error message neededMissing messageMissing message Error message" -> "Error message needed"
    perl -i -pe 's/"Error message neededMissing messageMissing message Error message"/"Error message needed"/g' "$file"
    
    # Fix pattern: "Failed to create AsyncCanonicalAbi Error message" -> "Failed to create AsyncCanonicalAbi"
    perl -i -pe 's/"Failed to create AsyncCanonicalAbi Error message"/"Failed to create AsyncCanonicalAbi"/g' "$file"
    
    # Fix pattern: "Expected values Error message" -> "Expected values"
    perl -i -pe 's/"Expected values Error message"/"Expected values"/g' "$file"
    
    # Fix pattern: "Expected immediate result Error message" -> "Expected immediate result"
    perl -i -pe 's/"Expected immediate result Error message"/"Expected immediate result"/g' "$file"
    
    # Fix pattern: "Expected stream result Error message" -> "Expected stream result"
    perl -i -pe 's/"Expected stream result Error message"/"Expected stream result"/g' "$file"
    
    # Fix pattern: "starting Error message" -> "starting"
    perl -i -pe 's/"starting Error message"/"starting"/g' "$file"
    
    # Fix pattern: "async-call Error message" -> "async-call"
    perl -i -pe 's/"async-call Error message"/"async-call"/g' "$file"
    
    # Fix pattern: "completed Error message" -> "completed"
    perl -i -pe 's/"completed Error message"/"completed"/g' "$file"
    
    # Fix pattern: "stream-read Error message" -> "stream-read"
    perl -i -pe 's/"stream-read Error message"/"stream-read"/g' "$file"
    
    # Fix pattern: "Expected timeout timer Error message" -> "Expected timeout timer"
    perl -i -pe 's/"Expected timeout timer Error message"/"Expected timeout timer"/g' "$file"
    
    # Fix pattern: "Failed to create default FuelDebtCreditSystem Error message" -> "Failed to create default FuelDebtCreditSystem"
    perl -i -pe 's/"Failed to create default FuelDebtCreditSystem Error message"/"Failed to create default FuelDebtCreditSystem"/g' "$file"
    
    # Fix pattern: "Expected Ready(Some(43)) Error message" -> "Expected Ready(Some(43))"
    perl -i -pe 's/"Expected Ready\(Some\(43\)\) Error message"/"Expected Ready(Some(43))"/g' "$file"
    
    # Fix pattern: "Expected Pending Error message" -> "Expected Pending"
    perl -i -pe 's/"Expected Pending Error message"/"Expected Pending"/g' "$file"
    
    # Fix pattern: "Channel full))" -> "Channel full"
    perl -i -pe 's/"Channel full\)\)"/"Channel full"/g' "$file"
    
    # Fix pattern: "Expected bounded channel Error message" -> "Expected bounded channel"
    perl -i -pe 's/"Expected bounded channel Error message"/"Expected bounded channel"/g' "$file"
    
    # Fix pattern: "test_data Error message" -> "test_data"
    perl -i -pe 's/"test_data Error message"/"test_data"/g' "$file"
    
    # Fix pattern: "Component not found Error message" -> "Component not found"
    perl -i -pe 's/"Component not found Error message"/"Component not found"/g' "$file"
    
    # Fix pattern: "test Error message" -> "test"
    perl -i -pe 's/"test Error message"/"test"/g' "$file"
    
    # Fix pattern: "FuelChain polled after completion Error message" -> "FuelChain polled after completion"
    perl -i -pe 's/"FuelChain polled after completion Error message"/"FuelChain polled after completion"/g' "$file"
    
    # Fix pattern: "hello Error message" -> "hello"
    perl -i -pe 's/"hello Error message"/"hello"/g' "$file"
    
    # Fix pattern: "asil-d Error message" -> "asil-d"
    perl -i -pe 's/"asil-d Error message"/"asil-d"/g' "$file"
    
    # Fix pattern: "asil-dMissing messageMissing message Error message" -> "asil-d"
    perl -i -pe 's/"asil-dMissing messageMissing message Error message"/"asil-d"/g' "$file"
    
    # Fix pattern: "\nStack trace:\n Error message" -> "\nStack trace:\n"
    perl -i -pe 's/\\nStack trace:\\n Error message/\\nStack trace:\\n/g' "$file"
    
    # Fix pattern: "TaskRegistry allocation should not fail in default construction Error message" -> "TaskRegistry allocation should not fail in default construction"
    perl -i -pe 's/"TaskRegistry allocation should not fail in default construction Error message"/"TaskRegistry allocation should not fail in default construction"/g' "$file"
    
    # Fix pattern: "Failed to acquire task registry lockMissing messageMissing message Error message" -> "Failed to acquire task registry lock"
    perl -i -pe 's/"Failed to acquire task registry lockMissing messageMissing message Error message"/"Failed to acquire task registry lock"/g' "$file"
    
    # Fix pattern: "Failed to spawn taskMissing messageMissing message Error message" -> "Failed to spawn task"
    perl -i -pe 's/"Failed to spawn taskMissing messageMissing message Error message"/"Failed to spawn task"/g' "$file"
    
    # Fix pattern: "Failed to spawn subtaskMissing messageMissing message Error message" -> "Failed to spawn subtask"
    perl -i -pe 's/"Failed to spawn subtaskMissing messageMissing message Error message"/"Failed to spawn subtask"/g' "$file"
    
    # Fix pattern: "\nError Context Chain:\n Error message" -> "\nError Context Chain:\n"
    perl -i -pe 's/\\nError Context Chain:\\n Error message/\\nError Context Chain:\\n/g' "$file"
    
    # Fix pattern: "Failed to create default manager Error message" -> "Failed to create default manager"
    perl -i -pe 's/"Failed to create default manager Error message"/"Failed to create default manager"/g' "$file"
    
    # Fix pattern: "Failed to create default priority inheritance protocol Error message" -> "Failed to create default priority inheritance protocol"
    perl -i -pe 's/"Failed to create default priority inheritance protocol Error message"/"Failed to create default priority inheritance protocol"/g' "$file"
    
    # Fix remaining corrupted expectation patterns with additional characters
    perl -i -pe 's/\.expect\("([^"]*?) Error message"\)/.expect("$1")/g' "$file"
    
    # Fix specific format! issue
    perl -i -pe 's/format!\(\),/format!(""),/g' "$file"
    
done

echo "Fixed remaining unterminated string literals in async files"