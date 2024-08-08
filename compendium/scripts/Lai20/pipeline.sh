esearch -db sra -query PRJNA528505 | efetch -format runinfo | cut -d "," -f 1 > SRR.numbers
