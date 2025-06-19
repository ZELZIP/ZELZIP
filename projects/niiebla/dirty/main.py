import libWiiPy
import json

wad = libWiiPy.title.WAD()
wad.load(open("test.wad", "rb").read())

ticket = libWiiPy.title.Ticket()
ticket.load(wad.get_ticket_data())
print(ticket.get_title_id())

tmd = libWiiPy.title.TMD()
tmd.load(wad.get_tmd_data())

title_key = ticket.get_title_key()

content_region = libWiiPy.title.ContentRegion()
content_region.load(wad.get_content_data(), tmd.content_records)

decrypted_content = content_region.get_content_by_index(0, title_key, skip_hash=True)

print(decrypted_content)
