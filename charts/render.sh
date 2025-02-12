#!/usr/bin/env bash

for file in *.drawio
do
  drawio -x -f png "$file" -o rendered/
done