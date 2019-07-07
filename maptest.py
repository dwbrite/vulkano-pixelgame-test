#!/usr/bin/env python3

import random

def create_array(x,y):
 array=[]
 for i in range(y):
  array.append([])
  for j in range(x):
   array[-1].append(random.randint(1,256))
 return array

print(create_array(20,20))

