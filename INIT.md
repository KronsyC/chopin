# Chopin Init process

---


This document aims to outline the initialization process used by the CHOPIN kernel




1. Stage 0 / Pre-Initialization


The Pre-Initialization phase of the kernel is solely responsible 
for correctly creating a basic environment for the next stage of the kernel 
namely finding kernel memory, and making sure the kernel is running on the right hart

2. Stage 1 / Initialization 

This stage is responsible for making the kernel runtime environment and initialization 
including full device tree reading, and loading the initramfs such that the minimum required providers 
like the filesystem, memory, etc. can be used
