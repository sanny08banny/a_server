# house_server
![WhatsApp Image 2023-10-05 at 7 19 22 PM](https://github.com/GithinjiHans/house_server/assets/92337850/31291cbf-d8fe-4648-849d-bc0518c36de3)
#
ukimake API request ya cars kwa home page, the server responds with an array of objects with that structure
#
![WhatsApp Image 2023-10-05 at 7 28 06 PM](https://github.com/GithinjiHans/house_server/assets/92337850/96a1763b-7be3-44f4-aae0-757b339dda97)
#
after this, you'll collect all the index 0 images from the car objects and put them in an array of objects that look like this and send them to the server(I'll provide the endpoint)....the example above requests for image one only of the individual cars with unique IDs.
#
that's how the images will be loaded kwa home page, with the captions from the first json file(the prices)
#
![WhatsApp Image 2023-10-05 at 7 41 39 PM](https://github.com/GithinjiHans/house_server/assets/92337850/1a1a8c21-7507-40e6-b75f-e55c7c1a4b4f)
#
user akiclick a certain car ataenda kwa car details page, where you'll make this request for all the images of that car...assuming umecache ile json juu ndio iko na the rest of the details...here the user will view all the car images and its details

All that ni for any user, logged in or not
juu hakuna app huboo kama yenye inakuforce ulogin ama ucreate account na we unataka tu kuona what they're offering
