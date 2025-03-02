#!/bin/bash

cargo pup 2>&1 | grep -v 'Finished' | diff - expected_output
