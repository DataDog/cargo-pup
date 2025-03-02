#!/bin/bash

cargo pup 2>&1 | grep -v 'Finished' > expected_output
