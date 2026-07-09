use water_http::WaterController;

WaterController! {
    holder -> data_holder::MainDataHolder,
    name -> ProfileController,
    functions -> {

        GET_edit_profile -> editProfile -> edit(context) async {

            _= context.send_str("edit").await;
        }
    }
}