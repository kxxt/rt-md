#!/usr/bin/env ruby

# translation pcaps to DNS log files

def translate(path)
    output = `tshark -r #{path} -2 -R 'dns.flags.response == 0' -t e`
    File.write("#{path}.log", output)
end

Dir.glob("raw/*/*.pcap").each do |pcap|
    if pcap.include? "0000" then
        next
    end
    puts "Translating #{pcap} to log"
    translate(pcap)
end
    