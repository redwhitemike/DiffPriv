#! /bin/bash

k=(3 20)
eps=(1 10 50)

function create_float() {
  echo "$1" | bc -l
}

for kanon in "${k[@]}"
do
  for e in "${eps[@]}"
  do
    java -jar moa-ppsm.jar "Anonymize -s (ArffFileStream -f datasets/adult_test.arff) -f (differentialprivacy.DifferentialPrivacyFilter -k $kanon -e 0$(create_float "$e/100") -b $(($kanon * 4 - 1))) -z -a $(echo "moa-output/output_$(echo $kanon)_0$(create_float "$e/100").csv") -r report.txt -e check"
  done;
done;
