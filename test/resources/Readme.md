
## Resources on understanding VCF Format

- http://vcftools.sourceforge.net/VCF-poster.pdf
- https://faculty.washington.edu/browning/intro-to-vcf.html#example
- Official specification for version 4.3: http://samtools.github.io/hts-specs/VCFv4.3.pdf

## Sample VCF files

- https://github.com/vcflib/vcflib/tree/master/samples
- https://www.internationalgenome.org/data

## BCFTOOLS

### References

- http://samtools.github.io/bcftools
- http://samtools.github.io/bcftools/bcftools.html#query

### Sample commands

GP Probabilities: 

`bcftools query -f '%CHROM\t%POS\t%REF\t%ALT\t[ %GP]\n' file.vcf`

Multiple format fields:

`bcftools query -f '%CHROM\t%POS\t%REF\t%ALT\tGT:[ %GT] \t GP:[ %GP]\n' file.vcf`

Filter based on ids (ids stored in a file called `ids.txt`) and store the result in a file called `out.txt`:

`bcftools query -i 'ID=@ids.txt' -f '%CHROM\t%POS\t%ID\t%REF\t%ALT\t[ %GP]\n' -o out.txt file.vcf`


## Selectively encrypting a VCF file without disrupting format

Consider the following example VCF file (call it file.vcf) and a scenario where we want to encrypt the
Genotype Posterior Probabilities (GP) values _only_ without compromising the format of the file. That is,
after encrypting GP values the file should still be parsable by, say, `bcftools`.

Original file (file.vcf):
```
##fileformat=VCFv4.3
##contig=<ID=20,length=62435964,assembly=B36,md5=f126cdf8a6e0c7f379d618ff66beb2da,species="Homo sapiens",taxonomy=x>
##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
##FORMAT=<ID=GP,Number=G,Type=Float,Description="Genotype Probabilities">
##FORMAT=<ID=PL,Number=G,Type=Float,Description="Phred-scaled Genotype Likelihoods">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	SAMP001	SAMP002
20	1291018	rs11449	G	A	.	PASS	.	GT	0/0	0/1
20	2300608	rs84825	C	T	.	PASS	.	GT:GP	0/1:.	0/1:0.03,0.97,0
20	2301308	rs84823	T	G	.	PASS	.	GT:PL	./.:.	1/1:10,5,0
```

### Attempt 1: Encrypt value and change nothing else:

For simplicity we will indicate ciphertext with the string "XXXXX". A string representation of a ciphertext can be
generated using, for instance, `base64` encoding.

Encrypted file (encrypted1.vcf):
 - GP probabilities where replaced with their encrypted value XXXXX.
```
##fileformat=VCFv4.3
##contig=<ID=20,length=62435964,assembly=B36,md5=f126cdf8a6e0c7f379d618ff66beb2da,species="Homo sapiens",taxonomy=x>
##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
##FORMAT=<ID=GP,Number=G,Type=Float,Description="Genotype Probabilities">
##FORMAT=<ID=PL,Number=G,Type=Float,Description="Phred-scaled Genotype Likelihoods">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	SAMP001	SAMP002
20	1291018	rs11449	G	A	.	PASS	.	GT	0/0	0/1
20	2300608	rs84825	C	T	.	PASS	.	GT:GP	0/1:XXXXX	0/1:XXXXX
20	2301308	rs84823	T	G	.	PASS	.	GT:PL	./.:.	1/1:10,5,0
```

Running the command `bcftools query -f '%ID\t[ %GP]\n' encrypted1.vcf` yields the following error:

```
[E::vcf_parse_format] Invalid character 'X' in 'GP' FORMAT field at 20:2300608
rs11449	 . .
rs84823	 . .
```

The error persists even when not explicitly printing GP, `bcftools query -f '%ID\t[ %PL]\n' encrypted1.vcf` yields the following error:
```
[E::vcf_parse_format] Invalid character 'X' in 'GP' FORMAT field at 20:2300608
rs11449	 . .
rs84823	 . 10,5,0
```

### Attempt 2: Encrypt value and change corresponding type:

Encrypted file (encrypted2.vcf):
 - GP probabilities where replaced with their encrypted value XXXXX.
 - Type of GP was changed from `Float` to `String`
```
##fileformat=VCFv4.3
##contig=<ID=20,length=62435964,assembly=B36,md5=f126cdf8a6e0c7f379d618ff66beb2da,species="Homo sapiens",taxonomy=x>
##FORMAT=<ID=GT,Number=1,Type=String,Description="Genotype">
##FORMAT=<ID=GP,Number=G,Type=String,Description="Genotype Probabilities">
##FORMAT=<ID=PL,Number=G,Type=Float,Description="Phred-scaled Genotype Likelihoods">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	SAMP001	SAMP002
20	1291018	rs11449	G	A	.	PASS	.	GT	0/0	0/1
20	2300608	rs84825	C	T	.	PASS	.	GT:GP	0/1:XXXXX	0/1:XXXXX
20	2301308	rs84823	T	G	.	PASS	.	GT:PL	./.:.	1/1:10,5,0
```

Running the command `bcftools query -f '%ID\t[ %GP]\n' encrypted2.vcf` yields the expected:

```
rs11449	 . .
rs84825	 . XXXXX
rs84823	 . .
```
