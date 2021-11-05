Based off of https://github.com/vn-input/ibus-unikey, 
https://github.com/phuang/ibus-tmpl and 
https://github.com/ibus/ibus/blob/master/src/ibusenginesimple.c

```shell
./test.sh
ibus restart
ibus engine eei
```

`ctrl+e` opens the emoji/symbol lookup table.
`ctrl+w` while in the middle of typing a word opens autocomplete for that word.

## Generating dictionary data
First, download the en_US hunspell dictionary data from http://wordlist.aspell.net/dicts/
```shell
sudo apt-get install hunspell-tools
unzip hunspell-en_US-2020.12.07.zip
unmunch en_US.dic en_US.aff > hunspell_US.txt
```

## Word frequency data
```shell
wget https://norvig.com/ngrams/count_1w.txt
```
