# house_server
![WhatsApp Image 2023-10-05 at 7 19 22 PM](https://github.com/GithinjiHans/house_server/assets/92337850/31291cbf-d8fe-4648-849d-bc0518c36de3)
<br>
ukimake API request ya cars kwa home page, the server responds with an array of objects with that structure
<br>
![WhatsApp Image 2023-10-05 at 7 28 06 PM](https://github.com/GithinjiHans/house_server/assets/92337850/96a1763b-7be3-44f4-aae0-757b339dda97)
<br>
after this, you'll collect all the index 0 images from the car objects and put them in an array of objects that look like this and send them to the server(I'll provide the endpoint)....the example above requests for image one only of the individual cars with unique IDs.
<br>
that's how the images will be loaded kwa home page, with the captions from the first json file(the prices)
<br>
![WhatsApp Image 2023-10-05 at 7 41 39 PM](https://github.com/GithinjiHans/house_server/assets/92337850/1a1a8c21-7507-40e6-b75f-e55c7c1a4b4f)
<br>
user akiclick a certain car ataenda kwa car details page, where you'll make this request for all the images of that car...assuming umecache ile json juu ndio iko na the rest of the details...here the user will view all the car images and its details

All that ni for any user, logged in or not
juu hakuna app huboo kama yenye inakuforce ulogin ama ucreate account na we unataka tu kuona what they're offering
<br>
![image](https://github.com/GithinjiHans/house_server/assets/92337850/73443dc7-62bf-4e49-b979-0b6d56ee2de9)
<br>
If someone chooses to pay via mpesa, the app should send the request in this format
<br>
![image](https://github.com/GithinjiHans/house_server/assets/92337850/b34e5d4b-362c-4534-88fb-73f97875c758)
if they choose to pay via the tokens, they'll send the request in this format, to the same endpoint as mpesa payment.
