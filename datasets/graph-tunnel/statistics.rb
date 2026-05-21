#!/usr/bin/env ruby

require 'set'
require 'public_suffix'

def extract_dns_queries(pcap_path)
    return `tshark -r #{pcap_path} -2 -R 'dns.flags.response == 0'`
end

def make_statistics(tshark_output)
    clients = Set.new
    servers = Set.new
    domains = Set.new
    queries = 0
    tshark_output.each_line do |line|
        number, timestamp, client, _, server, _, _, _, _, trans, qtype, domain = line.split()
        clients << client
        servers << server
        domains << PublicSuffix.domain(domain)
        queries += 1
    end
    return clients, servers, domains, queries
end

# print (extract_dns_queries "normal/normal/normal_00000_20230805150331.pcap")

Dir.glob("raw/**/*.pcap").each do |pcap|
    puts "---- BEGIN Statistics for #{pcap} ----"
    clients, servers, domains, queries = make_statistics (extract_dns_queries pcap)
    puts "Clients: #{clients}"
    puts "Servers: #{servers}"
    puts "Domains: #{domains}"
    puts "Queries: #{queries}"
    puts "---- END   Statistics for #{pcap} ----"
end
