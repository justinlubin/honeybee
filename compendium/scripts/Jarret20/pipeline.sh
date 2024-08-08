esearch -db sra -query PRJNA594861 \
  | efetch -format runinfo \
  | cut -d "," -f 1 \
  | tail -n +2 \
  > intermediates/Jarret20/SRR.numbers
