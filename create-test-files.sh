#!/bin/bash

# Create sample text files
echo "This is a sample text file for testing S3 uploads." > file1.txt
echo "Another sample text file with different content." > file2.txt
echo "A third sample text file for concurrent upload testing." > file3.txt

# Make the script executable
chmod +x create_test_files.sh

echo "Test files created successfully!"
