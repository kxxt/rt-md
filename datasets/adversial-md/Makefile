.PHONY: clean all

all: eval.csv.zst client.oracle all_clients val_all_domains.list

clean:
	rm -f eval.csv eval.csv.zst client.oracle all_clients val_all_domains.list

eval.csv: generate.py
	./generate.py > $@

all_clients: eval.csv
	tail -n +2 eval.csv | cut -d',' -f 2 | tr -d ' ' | sort | uniq > all_clients

eval.csv.zst: eval.csv
	zstd -f $< -o $@

val_all_domains.list client.oracle &: eval.csv.zst
	env -C ../.. SAVE_UNIQUE_DOMAINS=1 cargo run --bin gen-client-oracle -- "$(realpath ./eval.csv.zst)"