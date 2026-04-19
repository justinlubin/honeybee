if (!require("BiocManager", quietly = TRUE))
    install.packages("BiocManager")

BiocManager::install(version = "3.22")
BiocManager::install("tximport")
BiocManager::install("DESeq2")
BiocManager::install("rhdf5")

install.packages("devtools")
devtools::install_github("pachterlab/sleuth")
