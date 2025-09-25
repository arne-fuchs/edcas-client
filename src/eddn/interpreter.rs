use log::{error, info};
use postgres::Client;

use crate::eddn::edcas_error::EdcasError;

pub fn interpret_json(
    journal_id: i64,
    event: &str,
    json: serde_json::Value,
    client: &mut Client,
) -> Result<(), EdcasError> {
    use crate::edcas::event::*;

    match event {
        //Navigation
        "FSDJump" => {
            let fsdjump = match serde_json::from_value::<fsdjump::Fsdjump>(json) {
                Ok(fsdjump) => fsdjump,
                Err(err) => return Err(EdcasError::new(format!("[Fsdjump]: {}", err))),
            };
            match fsdjump.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("[Fsdjump]: {}", err))),
            }
            return Ok(());
        }
        "Location" => {
            let location = match serde_json::from_value::<location::Location>(json) {
                Ok(location) => location,
                Err(err) => return Err(EdcasError::new(format!("[Location]: {}", err))),
            };
            match location.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("[Location]: {}", err))),
            }
            return Ok(());
        }
        "CarrierJump" => match json.get("MarketID") {
            Some(_) => {
                let carrierjump = match serde_json::from_value::<carrierjump::Carrierjump>(json) {
                    Ok(carrierjump) => carrierjump,
                    Err(err) => return Err(EdcasError::new(format!("[Carrierjump]: {}", err))),
                };
                match carrierjump.insert_into_db(journal_id, client) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(EdcasError::new(format!("[Carrierjump]: {}", err))),
                }
                return Ok(());
            }
            None => {
                let carrierjump =
                    match serde_json::from_value::<carrierjump::Carrierjumponfoot>(json) {
                        Ok(carrierjump) => carrierjump,
                        Err(err) => {
                            return Err(EdcasError::new(format!("[Carrierjumponfoot]:{}", err)))
                        }
                    };
                match carrierjump.insert_into_db(journal_id, client) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(EdcasError::new(format!("[Carrierjump]:{}", err))),
                }
                return Ok(());
            }
        },
        "SupercruiseEntry" => {
            //Probably nothing
            info!("Registered SupercruiseEntry: {}", journal_id);
            return Ok(());
        }
        "SupercruiseExit" => {
            //Probably nothing
            info!("Registered SupercruiseExit: {}", journal_id);
            return Ok(());
        }
        "StartJump" => {
            //{ "timestamp":"2022-10-16T23:25:05Z", "event":"StartJump", "JumpType":"Hyperspace", "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K" }
            //Probably nothing
            info!("Registered StartJump: {}", journal_id);
            return Ok(());
        }
        //{ "timestamp":"2022-10-16T23:24:46Z", "event":"FSDTarget", "Name":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K", "RemainingJumpsInRoute":1 }
        "FSDTarget" => {
            //Probably nothing
            info!("Registered FSDTarget: {}", journal_id);
            return Ok(());
        } //If system has been targeted
        "NavRoute" => {
            //{"Route": [{"StarPos": [18606.21875, -64.09375, 33004.25], "StarClass": "M", "StarSystem": "Spase RA-V b36-27", "SystemAddress": 60294227188025}, {"StarPos": [18611.34375, -67.03125, 33005.25], "StarClass": "G", "StarSystem": "Spase IF-Y c17-4", "SystemAddress": 1214569720986}, {"StarPos": [18614.75, -67.75, 32998.59375], "StarClass": "M", "StarSystem": "Spase IF-Y c17-7", "SystemAddress": 2039203441818}, {"StarPos": [18618.28125, -68.34375, 32994.0], "StarClass": "K", "StarSystem": "Spase JF-Y c17-18", "SystemAddress": 5062927527066}, {"StarPos": [18620.03125, -72.21875, 32992.6875], "StarClass": "M", "StarSystem": "Spase QP-W b35-0", "SystemAddress": 920867658033}, {"StarPos": [18624.5, -73.78125, 32987.9375], "StarClass": "G", "StarSystem": "Spase JF-Y c17-79", "SystemAddress": 21830479850650}, {"StarPos": [18626.6875, -72.78125, 32985.65625], "StarClass": "Y", "StarSystem": "Spase JQ-R a72-0", "SystemAddress": 7367478915688}, {"StarPos": [18629.53125, -75.1875, 32980.625], "StarClass": "K", "StarSystem": "Spase JF-Y c17-88", "SystemAddress": 24304381013146}, {"StarPos": [18630.5625, -73.15625, 32977.0625], "StarClass": "M", "StarSystem": "Spase QP-W b35-28", "SystemAddress": 62493518813489}, {"StarPos": [18635.46875, -73.25, 32976.65625], "StarClass": "L", "StarSystem": "Spase GK-T a71-0", "SystemAddress": 7368552657504}, {"StarPos": [18636.375, -77.34375, 32976.875], "StarClass": "K", "StarSystem": "Spase JF-Y c17-13", "SystemAddress": 3688537992346}, {"StarPos": [18637.25, -80.28125, 32976.1875], "StarClass": "TTS", "StarSystem": "Spase IF-T a71-0", "SystemAddress": 7368552526432}, {"StarPos": [18640.65625, -82.09375, 32969.65625], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-6", "SystemAddress": 14115275626793}, {"StarPos": [18641.53125, -80.6875, 32965.84375], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-26", "SystemAddress": 58095740737833}, {"StarPos": [18641.96875, -81.75, 32961.8125], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-16", "SystemAddress": 36105508182313}, {"StarPos": [18646.6875, -82.09375, 32958.125], "StarClass": "T", "StarSystem": "Spase BT-W a69-1", "SystemAddress": 24961812312656}, {"StarPos": [18646.6875, -83.3125, 32956.5625], "StarClass": "Y", "StarSystem": "Spase BT-W a69-0", "SystemAddress": 7369626268240}, {"StarPos": [18650.84375, -84.21875, 32953.15625], "StarClass": "G", "StarSystem": "Spase FZ-Z c16-88", "SystemAddress": 24304381013138}, {"StarPos": [18653.34375, -81.46875, 32952.90625], "StarClass": "Y", "StarSystem": "Spase XM-Y a68-0", "SystemAddress": 7369626268232}, {"StarPos": [18659.125, -82.875, 32950.25], "StarClass": "K", "StarSystem": "Spase GZ-Z c16-95", "SystemAddress": 26228593470610}, {"StarPos": [18659.28125, -81.40625, 32948.6875], "StarClass": "M", "StarSystem": "Spase KD-A b34-10", "SystemAddress": 22911637084449}, {"StarPos": [18662.5625, -81.53125, 32941.9375], "StarClass": "K", "StarSystem": "Spase GZ-Z c16-65", "SystemAddress": 17982256262290}, {"StarPos": [18664.5, -78.875, 32938.625], "StarClass": "M", "StarSystem": "Spase KD-A b34-26", "SystemAddress": 58096009173281}, {"StarPos": [18669.5, -80.25, 32933.75], "StarClass": "M", "StarSystem": "Spase GX-B b33-11", "SystemAddress": 25110660339993}, {"StarPos": [18669.5625, -78.9375, 32930.71875], "StarClass": "K", "StarSystem": "Spase CT-B c16-90", "SystemAddress": 24854203935882}, {"StarPos": [18672.09375, -77.9375, 32924.875], "StarClass": "F", "StarSystem": "Spase MT-O d7-82", "SystemAddress": 2831901775427}, {"StarPos": [18675.21875, -80.3125, 32921.625], "StarClass": "K", "StarSystem": "Spase CT-B c16-0", "SystemAddress": 115192310922}, {"StarPos": [18677.65625, -80.375, 32920.21875], "StarClass": "M", "StarSystem": "Spase HX-B b33-28", "SystemAddress": 62494324119833}, {"StarPos": [18678.0, -78.4375, 32917.28125], "StarClass": "T", "StarSystem": "Spase OU-D a66-1", "SystemAddress": 24965033538096}, {"StarPos": [18682.0, -78.0625, 32917.09375], "StarClass": "M", "StarSystem": "Spase HX-B b33-22", "SystemAddress": 49300184586521}, {"StarPos": [18686.0, -83.875, 32915.53125], "StarClass": "Y", "StarSystem": "Spase PU-D a66-0", "SystemAddress": 7373921235504}, {"StarPos": [18689.6875, -86.125, 32911.75], "StarClass": "M", "StarSystem": "Spase FM-D b32-17", "SystemAddress": 38305068243217}, {"StarPos": [18690.125, -87.9375, 32908.78125], "StarClass": "F", "StarSystem": "Spase MT-O d7-26", "SystemAddress": 907756426819}, {"StarPos": [18691.28125, -85.75, 32905.21875], "StarClass": "M", "StarSystem": "Spase CT-B c16-77", "SystemAddress": 21280791145610}, {"StarPos": [18695.09375, -90.25, 32901.0625], "StarClass": "F", "StarSystem": "Spase MT-O d7-89", "SystemAddress": 3072419944003}, {"StarPos": [18695.53125, -89.75, 32899.1875], "StarClass": "M", "StarSystem": "Spase GM-D b32-12", "SystemAddress": 27310220400913}, {"StarPos": [18697.53125, -94.875, 32895.46875], "StarClass": "TTS", "StarSystem": "Spase KD-H a64-0", "SystemAddress": 7374994846240}, {"StarPos": [18698.75, -97.59375, 32892.5], "StarClass": "K", "StarSystem": "Spase ZM-D c15-61", "SystemAddress": 16882811743362}, {"StarPos": [18701.4375, -98.9375, 32888.21875], "StarClass": "M", "StarSystem": "Spase ZM-D c15-60", "SystemAddress": 16607933836418}, {"StarPos": [18704.875, -100.375, 32885.53125], "StarClass": "K", "StarSystem": "Spase ZM-D c15-57", "SystemAddress": 15783300115586}, {"StarPos": [18705.625, -102.5625, 32883.1875], "StarClass": "M", "StarSystem": "Spase CG-F b31-19", "SystemAddress": 42703383189769}, {"StarPos": [18708.65625, -104.71875, 32880.09375], "StarClass": "G", "StarSystem": "Spase MT-O d7-221", "SystemAddress": 7607905408579}, {"StarPos": [18708.375, -104.9375, 32876.03125], "StarClass": "M", "StarSystem": "Spase CG-F b31-26", "SystemAddress": 58096545978633}, {"StarPos": [18710.28125, -105.125, 32873.71875], "StarClass": "M", "StarSystem": "Spase BI-D c15-44", "SystemAddress": 12209887292546}, {"StarPos": [18710.96875, -105.71875, 32868.03125], "StarClass": "M", "StarSystem": "Spase AV-G b30-5", "SystemAddress": 11917057546497}, {"StarPos": [18712.09375, -109.46875, 32867.15625], "StarClass": "M", "StarSystem": "Spase AV-G b30-6", "SystemAddress": 14116080802049}, {"StarPos": [18715.5, -110.34375, 32863.53125], "StarClass": "M", "StarSystem": "Spase BV-G b30-2", "SystemAddress": 5320256215297}, {"StarPos": [18716.15625, -110.0625, 32862.15625], "StarClass": "M", "StarSystem": "Spase BV-G b30-5", "SystemAddress": 11917325981953}, {"StarPos": [18719.875, -111.21875, 32856.71875], "StarClass": "F", "StarSystem": "Spase OO-O d7-133", "SystemAddress": 4584248415811}, {"StarPos": [18720.1875, -112.8125, 32854.90625], "StarClass": "M", "StarSystem": "Spase XO-I b29-6", "SystemAddress": 14116349237497}, {"StarPos": [18721.28125, -115.6875, 32851.03125], "StarClass": "F", "StarSystem": "Spase KI-Q d6-172", "SystemAddress": 5924278212155}, {"StarPos": [18721.53125, -118.1875, 32851.25], "StarClass": "M", "StarSystem": "Spase XO-I b29-20", "SystemAddress": 44902674815225}, {"StarPos": [18721.71875, -122.5, 32846.0], "StarClass": "Y", "StarSystem": "Spase YJ-P a59-0", "SystemAddress": 7377141936632}, {"StarPos": [18724.03125, -119.3125, 32841.625], "StarClass": "A", "StarSystem": "Spase KI-Q d6-114", "SystemAddress": 3931413386811}, {"StarPos": [18728.4375, -119.34375, 32838.875], "StarClass": "M", "StarSystem": "Spase XO-I b29-17", "SystemAddress": 38305605048569}, {"StarPos": [18732.5625, -119.25, 32833.96875], "StarClass": "M", "StarSystem": "Spase TI-K b28-13", "SystemAddress": 29509512026353}, {"StarPos": [18733.03125, -119.96875, 32831.15625], "StarClass": "F", "StarSystem": "Spase KI-Q d6-207", "SystemAddress": 7126869055035}, {"StarPos": [18735.28125, -122.3125, 32827.40625], "StarClass": "Y", "StarSystem": "Spase SX-S a57-0", "SystemAddress": 7379289420264}, {"StarPos": [18735.28125, -124.0625, 32825.09375], "StarClass": "T", "StarSystem": "Spase SX-S a57-1", "SystemAddress": 24971475464680}, {"StarPos": [18739.125, -126.875, 32825.84375], "StarClass": "K", "StarSystem": "Spase YB-F c14-50", "SystemAddress": 13859221843066}, {"StarPos": [18740.8125, -127.90625, 32821.8125], "StarClass": "T", "StarSystem": "Spase QM-U a56-0", "SystemAddress": 7379289289184}, {"StarPos": [18742.0625, -128.34375, 32822.3125], "StarClass": "K", "StarSystem": "Spase YB-F c14-69", "SystemAddress": 19081902075002}, {"StarPos": [18747.96875, -131.3125, 32819.1875], "StarClass": "M", "StarSystem": "Spase WD-K b28-9", "SystemAddress": 20713687374065}, {"StarPos": [18753.03125, -129.5625, 32813.84375], "StarClass": "G", "StarSystem": "Spase UV-G c13-27", "SystemAddress": 7537029983346}, {"StarPos": [18753.875, -128.8125, 32811.96875], "StarClass": "F", "StarSystem": "Spase LI-Q d6-25", "SystemAddress": 873413449275}, {"StarPos": [18754.21875, -123.3125, 32808.53125], "StarClass": "G", "StarSystem": "Spase LI-Q d6-266", "SystemAddress": 9154110395963}, {"StarPos": [18756.75, -123.125, 32806.25], "StarClass": "M", "StarSystem": "Spase RC-M b27-20", "SystemAddress": 44903211686121}, {"StarPos": [18757.6875, -123.4375, 32802.96875], "StarClass": "M", "StarSystem": "Spase RC-M b27-16", "SystemAddress": 36107118663913}, {"StarPos": [18761.3125, -122.34375, 32796.25], "StarClass": "T", "StarSystem": "Spase IF-Y a54-0", "SystemAddress": 7381436903888}, {"StarPos": [18764.25, -126.65625, 32793.5625], "StarClass": "M", "StarSystem": "Spase PR-N b26-12", "SystemAddress": 27311025576161}, {"StarPos": [18765.8125, -126.625, 32793.78125], "StarClass": "M", "StarSystem": "Spase PR-N b26-11", "SystemAddress": 25112002320609}, {"StarPos": [18767.71875, -125.625, 32786.71875], "StarClass": "M", "StarSystem": "Spase PR-N b26-2", "SystemAddress": 5320793020641}, {"StarPos": [18771.15625, -125.8125, 32783.40625], "StarClass": "K", "StarSystem": "Spase PR-N b26-1", "SystemAddress": 3121769765089}, {"StarPos": [18774.28125, -121.15625, 32780.0], "StarClass": "F", "StarSystem": "Spase LI-Q d6-229", "SystemAddress": 7882800076347}, {"StarPos": [18779.3125, -119.78125, 32778.875], "StarClass": "K", "StarSystem": "Spase VV-G c13-71", "SystemAddress": 19631724997746}, {"StarPos": [18784.875, -120.90625, 32774.78125], "StarClass": "M", "StarSystem": "Spase KQ-P b25-5", "SystemAddress": 11918131288281}, {"StarPos": [18786.90625, -118.78125, 32772.28125], "StarClass": "M", "StarSystem": "Spase KQ-P b25-4", "SystemAddress": 9719108032729}, {"StarPos": [18790.5625, -118.3125, 32767.25], "StarClass": "M", "StarSystem": "Spase KQ-P b25-20", "SystemAddress": 44903480121561}, {"StarPos": [18793.4375, -117.125, 32764.75], "StarClass": "Y", "StarSystem": "Spase VG-F a51-0", "SystemAddress": 7384658129328}, {"StarPos": [18796.90625, -116.78125, 32761.4375], "StarClass": "K", "StarSystem": "Spase RP-I c12-59", "SystemAddress": 16333190114410}, {"StarPos": [18799.25, -117.1875, 32758.25], "StarClass": "G", "StarSystem": "Spase HC-S d5-187", "SystemAddress": 6439691064883}, {"StarPos": [18798.375, -117.90625, 32753.5625], "StarClass": "M", "StarSystem": "Spase HK-R b24-8", "SystemAddress": 18515469490385}, {"StarPos": [18801.3125, -119.9375, 32749.09375], "StarClass": "M", "StarSystem": "Spase HK-R b24-18", "SystemAddress": 40505702045905}, {"StarPos": [18805.46875, -117.96875, 32744.09375], "StarClass": "M", "StarSystem": "Spase HK-R b24-12", "SystemAddress": 27311562512593}, {"StarPos": [18806.96875, -120.1875, 32741.53125], "StarClass": "M", "StarSystem": "Spase HK-R b24-6", "SystemAddress": 14117422979281}, {"StarPos": [18810.3125, -119.96875, 32739.375], "StarClass": "K", "StarSystem": "Spase RP-I c12-31", "SystemAddress": 8636608719978}, {"StarPos": [18814.6875, -120.59375, 32737.875], "StarClass": "M", "StarSystem": "Spase HK-R b24-3", "SystemAddress": 7520353212625}, {"StarPos": [18817.53125, -121.4375, 32736.84375], "StarClass": "M", "StarSystem": "Spase IK-R b24-1", "SystemAddress": 3122575136977}, {"StarPos": [18820.4375, -119.96875, 32731.65625], "StarClass": "M", "StarSystem": "Spase OJ-K c11-45", "SystemAddress": 12484966526050}, {"StarPos": [18818.96875, -120.75, 32730.21875], "StarClass": "K", "StarSystem": "Spase OJ-K c11-32", "SystemAddress": 8911553735778}, {"StarPos": [18822.5625, -121.28125, 32723.15625], "StarClass": "K", "StarSystem": "Spase OJ-K c11-50", "SystemAddress": 13859356060770}, {"StarPos": [18827.15625, -121.875, 32721.625], "StarClass": "TTS", "StarSystem": "Spase EE-T b23-21", "SystemAddress": 47103040248009}, {"StarPos": [18827.28125, -125.40625, 32720.84375], "StarClass": "F", "StarSystem": "Spase IC-S d5-140", "SystemAddress": 4824800138803}, {"StarPos": [18825.9375, -126.78125, 32715.1875], "StarClass": "A", "StarSystem": "Spase IC-S d5-103", "SystemAddress": 3553489819187}, {"StarPos": [18830.90625, -124.59375, 32710.28125], "StarClass": "A", "StarSystem": "Spase IC-S d5-172", "SystemAddress": 5924311766579}, {"StarPos": [18834.78125, -122.53125, 32708.4375], "StarClass": "M", "StarSystem": "Spase AY-U b22-4", "SystemAddress": 9719644903617}, {"StarPos": [18837.625, -123.53125, 32704.28125], "StarClass": "M", "StarSystem": "Spase BY-U b22-4", "SystemAddress": 9719913339073}, {"StarPos": [18837.75, -124.125, 32699.5], "StarClass": "M", "StarSystem": "Spase BY-U b22-2", "SystemAddress": 5321866827969}, {"StarPos": [18841.875, -126.96875, 32693.28125], "StarClass": "M", "StarSystem": "Spase ZM-W b21-1", "SystemAddress": 3122843506873}, {"StarPos": [18846.21875, -128.03125, 32692.5], "StarClass": "K", "StarSystem": "Spase KD-M c10-65", "SystemAddress": 17982524664922}, {"StarPos": [18846.21875, -129.28125, 32692.0], "StarClass": "M", "StarSystem": "Spase ZM-W b21-7", "SystemAddress": 16316983040185}, {"StarPos": [18850.15625, -131.40625, 32691.0625], "StarClass": "F", "StarSystem": "Spase EW-T d4-143", "SystemAddress": 4927879353899}, {"StarPos": [18849.71875, -136.5, 32689.84375], "StarClass": "K", "StarSystem": "Spase KD-M c10-33", "SystemAddress": 9186431642714}, {"StarPos": [18850.5625, -141.15625, 32686.875], "StarClass": "G", "StarSystem": "Spase EW-T d4-15", "SystemAddress": 529832842795}, {"StarPos": [18850.40625, -141.1875, 32682.78125], "StarClass": "M", "StarSystem": "Spase ZM-W b21-15", "SystemAddress": 33909169084601}, {"StarPos": [18850.875, -141.59375, 32679.65625], "StarClass": "M", "StarSystem": "Spase ZM-W b21-12", "SystemAddress": 27312099317945}, {"StarPos": [18850.1875, -143.0625, 32676.53125], "StarClass": "F", "StarSystem": "Spase EW-T d4-19", "SystemAddress": 667271796267}, {"StarPos": [18849.875, -139.0, 32672.59375], "StarClass": "M", "StarSystem": "Spase VG-Y b20-5", "SystemAddress": 11918936529073}, {"StarPos": [18854.0625, -139.0625, 32666.125], "StarClass": "M", "StarSystem": "Spase VG-Y b20-3", "SystemAddress": 7520890017969}, {"StarPos": [18857.40625, -137.46875, 32660.3125], "StarClass": "M", "StarSystem": "Spase WG-Y b20-2", "SystemAddress": 5322135197873}, {"StarPos": [18858.34375, -135.5625, 32655.5], "StarClass": "M", "StarSystem": "Spase WG-Y b20-0", "SystemAddress": 924088686769}, {"StarPos": [18862.8125, -139.78125, 32652.71875], "StarClass": "K", "StarSystem": "Spase HX-N c9-41", "SystemAddress": 11385522007122}, {"StarPos": [18864.0625, -142.46875, 32647.03125], "StarClass": "M", "StarSystem": "Spase SA-A b20-10", "SystemAddress": 22914321242281}, {"StarPos": [18868.40625, -141.90625, 32643.625], "StarClass": "M", "StarSystem": "Spase SA-A b20-11", "SystemAddress": 25113344497833}, {"StarPos": [18870.21875, -140.8125, 32639.15625], "StarClass": "M", "StarSystem": "Spase SA-A b20-12", "SystemAddress": 27312367753385}, {"StarPos": [18874.34375, -141.96875, 32635.90625], "StarClass": "K", "StarSystem": "Spase HX-N c9-77", "SystemAddress": 21281126657106}, {"StarPos": [18875.3125, -141.84375, 32635.4375], "StarClass": "M", "StarSystem": "Spase TA-A b20-2", "SystemAddress": 5322403633321}, {"StarPos": [18877.375, -140.875, 32636.34375], "StarClass": "K", "StarSystem": "Spase HX-N c9-2", "SystemAddress": 665283636306}, {"StarPos": [18880.9375, -141.0625, 32635.5625], "StarClass": "M", "StarSystem": "Spase TA-A b20-0", "SystemAddress": 924357122217}, {"StarPos": [18882.25, -141.6875, 32633.71875], "StarClass": "K", "StarSystem": "Spase HX-N c9-71", "SystemAddress": 19631859215442}, {"StarPos": [18885.28125, -140.6875, 32630.28125], "StarClass": "M", "StarSystem": "Spase PU-B b19-5", "SystemAddress": 11919473399969}, {"StarPos": [18885.78125, -138.53125, 32629.625], "StarClass": "M", "StarSystem": "Spase PU-B b19-4", "SystemAddress": 9720450144417}, {"StarPos": [18885.15625, -137.25, 32626.625], "StarClass": "A", "StarSystem": "Spase EW-T d4-68", "SystemAddress": 2350898976299}, {"StarPos": [18885.09375, -136.0625, 32621.8125], "StarClass": "M", "StarSystem": "Spase PU-B b19-11", "SystemAddress": 25113612933281}, {"StarPos": [18888.96875, -137.34375, 32619.4375], "StarClass": "M", "StarSystem": "Spase PU-B b19-15", "SystemAddress": 33909705955489}, {"StarPos": [18895.875, -134.5625, 32616.34375], "StarClass": "T", "StarSystem": "Spase EU-D a38-0", "SystemAddress": 7396469158208}, {"StarPos": [18897.84375, -137.28125, 32615.21875], "StarClass": "K", "StarSystem": "Spase IX-N c9-2", "SystemAddress": 665350745170}, {"StarPos": [18898.0, -139.59375, 32615.59375], "StarClass": "M", "StarSystem": "Spase QU-B b19-0", "SystemAddress": 924625557665}, {"StarPos": [18900.03125, -140.9375, 32615.25], "StarClass": "L", "StarSystem": "Spase QU-B b19-1", "SystemAddress": 3123648813217}, {"StarPos": [18899.9375, -143.4375, 32609.6875], "StarClass": "M", "StarSystem": "Spase MO-D b18-12", "SystemAddress": 27312904624281}, {"StarPos": [18903.53125, -143.59375, 32601.84375], "StarClass": "K", "StarSystem": "Spase ER-P c8-59", "SystemAddress": 16333391440970}, {"StarPos": [18905.625, -142.78125, 32598.6875], "StarClass": "K", "StarSystem": "Spase ER-P c8-56", "SystemAddress": 15508757720138}, {"StarPos": [18907.3125, -145.0, 32597.90625], "StarClass": "K", "StarSystem": "Spase ER-P c8-58", "SystemAddress": 16058513534026}, {"StarPos": [18906.1875, -147.0, 32594.09375], "StarClass": "K", "StarSystem": "Spase GM-P c8-47", "SystemAddress": 13034856524874}, {"StarPos": [18907.25, -149.25, 32589.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-19", "SystemAddress": 5338275130442}, {"StarPos": [18908.375, -152.5, 32585.0625], "StarClass": "M", "StarSystem": "Spase GM-P c8-21", "SystemAddress": 5888030944330}, {"StarPos": [18911.625, -151.71875, 32582.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-61", "SystemAddress": 16883147222090}, {"StarPos": [18912.84375, -152.6875, 32580.0], "StarClass": "A", "StarSystem": "Spase BQ-V d3-113", "SystemAddress": 3897103980067}, {"StarPos": [18917.0, -153.875, 32579.28125], "StarClass": "M", "StarSystem": "Spase LD-F b17-5", "SystemAddress": 11920010205329}, {"StarPos": [18917.78125, -152.4375, 32578.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-23", "SystemAddress": 6437786758218}, {"StarPos": [18922.21875, -150.96875, 32577.46875], "StarClass": "M", "StarSystem": "Spase LD-F b17-3", "SystemAddress": 7521963694225}, {"StarPos": [18922.0, -147.53125, 32577.25], "StarClass": "M", "StarSystem": "Spase LD-F b17-14", "SystemAddress": 31711219505297}, {"StarPos": [18923.5, -146.40625, 32572.4375], "StarClass": "G", "StarSystem": "Spase CG-R c7-34", "SystemAddress": 9461443734594}, {"StarPos": [18926.28125, -147.78125, 32570.03125], "StarClass": "M", "StarSystem": "Spase HX-G b16-12", "SystemAddress": 27313172994185}, {"StarPos": [18930.21875, -153.15625, 32571.71875], "StarClass": "F", "StarSystem": "Spase BQ-V d3-293", "SystemAddress": 10081856886307}, {"StarPos": [18932.28125, -153.78125, 32565.125], "StarClass": "M", "StarSystem": "Spase HX-G b16-9", "SystemAddress": 20716103227529}, {"StarPos": [18933.15625, -158.15625, 32560.71875], "StarClass": "M", "StarSystem": "Spase HX-G b16-14", "SystemAddress": 31711219505289}, {"StarPos": [18931.375, -157.5, 32558.28125], "StarClass": "M", "StarSystem": "Spase HX-G b16-5", "SystemAddress": 11920010205321}, {"StarPos": [18927.53125, -156.46875, 32555.59375], "StarClass": "M", "StarSystem": "Spase HX-G b16-10", "SystemAddress": 22915126483081}, {"StarPos": [18927.0, -157.34375, 32552.4375], "StarClass": "K", "StarSystem": "Spase CG-R c7-43", "SystemAddress": 11935344897090}, {"StarPos": [18929.09375, -156.25, 32546.8125], "StarClass": "F", "StarSystem": "Spase BQ-V d3-103", "SystemAddress": 3553506596387}, {"StarPos": [18929.65625, -158.53125, 32544.625], "StarClass": "M", "StarSystem": "Spase DR-I b15-6", "SystemAddress": 14119033460865}, {"StarPos": [18929.625, -161.90625, 32542.375], "StarClass": "M", "StarSystem": "Spase DR-I b15-8", "SystemAddress": 18517079971969}, {"StarPos": [18933.34375, -163.5, 32543.15625], "StarClass": "K", "StarSystem": "Spase CG-R c7-50", "SystemAddress": 13859490245698}, {"StarPos": [18939.3125, -161.5625, 32538.875], "StarClass": "K", "StarSystem": "Spase DG-R c7-45", "SystemAddress": 12485167819842}, {"StarPos": [18944.125, -165.46875, 32534.09375], "StarClass": "F", "StarSystem": "Spase XJ-X d2-273", "SystemAddress": 9394662118939}, {"StarPos": [18945.875, -164.96875, 32531.65625], "StarClass": "M", "StarSystem": "Spase AL-K b14-16", "SystemAddress": 36109534451833}, {"StarPos": [18947.4375, -162.28125, 32526.8125], "StarClass": "G", "StarSystem": "Spase ZZ-S c6-1", "SystemAddress": 390539914298}, {"StarPos": [18944.25, -159.46875, 32523.03125], "StarClass": "M", "StarSystem": "Spase AL-K b14-0", "SystemAddress": 925162363001}], "event": "NavRoute", "odyssey": true, "horizons": true, "timestamp": "2025-09-08T16:23:42Z"}
            //TODO What to do with NavRoute?
            return Ok(());
        }
        "NavRouteClear" => {
            info!("Registered FSDTarget: {}", journal_id);
            return Ok(());
        } //If navigation is complete -> no further information

        //Approaching
        "ApproachSettlement" => {
            //{"Name": "Bevis Foundry", "event": "ApproachSettlement", "BodyID": 11, "StarPos": [517.625, 45.5, 3351.65625], "odyssey": false, "BodyName": "Smojai JR-N d6-35 A 3 a", "Latitude": 30.852997, "MarketID": 4284372995, "horizons": true, "Longitude": 148.809814, "timestamp": "2025-09-08T16:23:36Z", "StarSystem": "Smojai JR-N d6-35", "SystemAddress": 1213185657531, "StationEconomy": "$economy_Industrial;", "StationFaction": {"Name": "Bot Network"}, "StationServices": ["dock", "autodock", "commodities", "contacts", "missions", "outfitting", "rearm", "refuel", "repair", "engineer", "facilitator", "flightcontroller", "stationoperations", "powerplay", "searchrescue", "stationMenu", "shop", "livery", "socialspace", "registeringcolonisation"], "StationEconomies": [{"Name": "$economy_Industrial;", "Proportion": 1.7}, {"Name": "$economy_Agri;", "Proportion": 0.2}, {"Name": "$economy_Refinery;", "Proportion": 0.2}, {"Name": "$economy_Extraction;", "Proportion": 0.05}], "StationAllegiance": "Federation", "StationGovernment": "$government_Corporate;"}
            //TODO Implement
        }
        "ApproachBody" => {
            info!("Registered ApproachBody: {}", journal_id);
            return Ok(());
        }
        "LeaveBody" => {
            info!("Registered LeaveBody: {}", journal_id);
            return Ok(());
        }
        "Liftoff" => {
            info!("Registered Liftoff: {}", journal_id);
            return Ok(());
        }
        "Touchdown" => {
            info!("Registered Touchdown: {}", journal_id);
            return Ok(());
        }
        "Embark" => {
            info!("Registered Embark: {}", journal_id);
            return Ok(());
        }
        "Disembark" => {
            info!("Registered Disembark: {}", journal_id);
            return Ok(());
        }

        //Scanning
        "DiscoveryScan" => {
            info!("Registered DiscoveryScan: {}", journal_id);
            return Ok(());
        }
        "FSSAllBodiesFound" => {
            //{"Count": 36, "event": "FSSAllBodiesFound", "StarPos": [581.65625, 154.8125, -189.8125], "odyssey": true, "horizons": true, "timestamp": "2025-09-08T16:24:12Z", "SystemName": "Wregoe ST-I d9-10", "SystemAddress": 354209007955}
            //TODO Does it make sense to save the body count?
            return Ok(());
        }
        "FSSDiscoveryScan" => {
            //{ "timestamp":"2022-10-16T23:46:48Z", "event":"FSSDiscoveryScan", "Progress":0.680273, "BodyCount":21, "NonBodyCount":80, "SystemName":"Ogmar", "SystemAddress":84180519395914 }
            //TODO Does it make sense to save the body + non-body count?
            return Ok(());
        } //Honk
        "FSSBodySignals" => {
            //TODO Save body signals Implement
        }
        "SAASignalsFound" => {
            //TODO Save body signals Implement
        }
        "FSSSignalDiscovered" => {
            //language=json
            let _ = r#"
            {
              "event": "FSSSignalDiscovered",
              "StarPos": [
                -22.28125,
                -5.8125,
                -28.09375
              ],
              "odyssey": true,
              "signals": [
                {
                  "IsStation": true,
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "Giacconi Sanctuary",
                  "SignalType": "StationCoriolis"
                },
                {
                  "IsStation": true,
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "KILO FOXTROT K7N-0TB",
                  "SignalType": "FleetCarrier"
                },
                {
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "$MULTIPLAYER_SCENARIO42_TITLE;",
                  "SignalType": "NavBeacon"
                }
              ],
              "horizons": true,
              "timestamp": "2025-09-08T17:49:26Z",
              "StarSystem": "Theta Persei",
              "SystemAddress": 1453586385251
            }
            "#;
            //TODO Does it make sense to save something here?
            return Ok(());
        }
        "SAAScanComplete" => {
            info!("Registered SAAScanComplete: {}", journal_id);
            return Ok(());
        }
        "Scan" => {
            if json.get("StarType").is_some() {
                //Star
                let star = match serde_json::from_value::<crate::edcas::assets::star::Star>(json) {
                    Ok(star) => star,
                    Err(err) => return Err(EdcasError::new(format!("[Star]: {}", err))),
                };
                match star.insert_into_db(journal_id, client) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(EdcasError::new(format!("[Star]: {}", err))),
                }
            } else {
                if let Some(scan_type) = json.get("ScanType") {
                    let scan_type = scan_type.as_str();
                    if let Some(scan_type) = scan_type {
                        match scan_type {
                            "AutoScan" | "NavBeaconDetail" => {
                                return Err(EdcasError::unimplemented())
                            }
                            "Basic" => return Err(EdcasError::unimplemented()),
                            _ => {}
                        }
                    }
                }
                //Body
                let body = match serde_json::from_value::<crate::edcas::assets::body::Body>(json) {
                    Ok(body) => body,
                    Err(err) => return Err(EdcasError::new(format!("[Body]: {}", err))),
                };
                match body.insert_into_db(journal_id, client) {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(EdcasError::new(format!("[Body]: {}", err))),
                }
            }

            return Ok(());
        }
        "ScanBaryCentre" => {
            //Planet scan with fss
            //TODO probably won't need it?
        }

        //Maintenance
        "RefuelAll" => {
            info!("Registered RefuelAll: {}", journal_id);
            return Ok(());
        }
        "Resupply" => {
            info!("Registered Resupply: {}", journal_id);
            return Ok(());
        }
        "Repair" => {
            info!("Registered Repair: {}", journal_id);
            return Ok(());
        }
        "BuyDrones" => {
            info!("Registered BuyDrones: {}", journal_id);
            return Ok(());
        }
        "SellDrones" => {
            info!("Registered SellDrones: {}", journal_id);
            return Ok(());
        }
        "BuyAmmo" => {
            info!("Registered BuyAmmo: {}", journal_id);
            return Ok(());
        }
        "ReservoirReplenished" => {
            info!("Registered ReservoirReplenished: {}", journal_id);
            return Ok(());
        }
        "RepairAll" => {
            info!("Registered RepairAll: {}", journal_id);
            return Ok(());
        }
        "RebootRepair" => {
            info!("Registered RebootRepair: {}", journal_id);
            return Ok(());
        }
        "RestockVehicle" => {
            info!("Registered RestockVehicle: {}", journal_id);
            return Ok(());
        }

        //Docking
        "DockingRequested" => {
            info!("Registered DockingRequested: {}", journal_id);
            return Ok(());
        }
        "DockingGranted" => {
            //Probably nothing
            return Ok(());
        }
        "Docked" => {
            let docked = match serde_json::from_value::<crate::edcas::event::docked::Docked>(json) {
                Ok(docked) => docked,
                Err(err) => return Err(EdcasError::new(format!("[Docked]: {}", err))),
            };
            match docked.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("[Docked]: {}", err))),
            }
        }
        "Undocked" => {
            info!("Registered Undocked: {}", journal_id);
            return Ok(());
        }

        //Engineer
        "EngineerProgress" => {
            info!("Registered EngineerProgress: {}", journal_id);
            return Ok(());
        }
        "EngineerCraft" => {
            info!("Registered EngineerCraft: {}", journal_id);
            return Ok(());
        }
        "EngineerContribution" => {
            info!("Registered EngineerContribution: {}", journal_id);
            return Ok(());
        }

        //Ship management
        "Shipyard" => {
            info!("Registered Shipyard: {}", journal_id);
            return Ok(());
        }
        "StoredShips" => {
            info!("Registered StoredShips: {}", journal_id);
            return Ok(());
        }
        "ShipyardSwap" => {
            info!("Registered ShipyardSwap: {}", journal_id);
            return Ok(());
        }
        "ShipLocker" => {
            info!("Registered ShipLocker: {}", journal_id);
            return Ok(());
        }
        "ModuleBuy" => {
            info!("Registered ModuleBuy: {}", journal_id);
            return Ok(());
        }
        "Outfitting" => {
            info!("Registered Outfitting: {}", journal_id);
            return Ok(());
        }
        "ModuleInfo" => {
            info!("Registered ModuleInfo: {}", journal_id);
            return Ok(());
        }
        "StoredModules" => {
            info!("Registered StoredModules: {}", journal_id);
            return Ok(());
        }
        "DockingCancelled" => {
            info!("Registered DockingCancelled: {}", journal_id);
            return Ok(());
        }
        "ShipyardBuy" => {
            info!("Registered ShipyardBuy: {}", journal_id);
            return Ok(());
        }
        "ShipyardNew" => {
            info!("Registered ShipyardNew: {}", journal_id);
            return Ok(());
        }
        "ShipyardTransfer" => {}
        "ModuleStore" => {}
        "ModuleSell" => {}
        "ModuleSellRemote" => {}
        "ModuleSwap" => {}

        //On foot
        "Backpack" => {}
        "BackpackChange" => {}
        "CollectItems" => {}
        "UpgradeSuit" => {}
        "Loadout" => {}
        "LoadoutEquipModule" => {}
        "SuitLoadout" => {}
        "UseConsumable" => {}
        "ScanOrganic" => {}
        "BuyWeapon" => {}

        //Market
        "MarketBuy" => {}
        "Market" => {}
        "MarketSell" => {}

        //SRV
        "LaunchSRV" => {}
        "DockSRV" => {}

        //Ship fight
        "ShipTargeted" => {}
        "UnderAttack" => {}
        "ShieldState" => {}
        "HullDamage" => {}

        "Materials" => {}
        "Cargo" => {}
        "MaterialCollected" => {}
        "Synthesis" => {}
        "EjectCargo" => {}
        "DropItems" => {}
        "LaunchDrone" => {}
        "MiningRefined" => {}
        "ProspectedAsteroid" => {}
        "CargoTransfer" => {}
        "CollectCargo" => {}

        //Mission and Redeeming
        "Missions" => {}
        "MissionAccepted" => {}
        "MissionRedirected" => {}
        "MissionCompleted" => {}
        "RedeemVoucher" => {}
        "Bounty" => {}
        "NpcCrewPaidWage" => {}
        "PayFines" => {}
        "MissionAbandoned" => {}
        "MissionFailed" => {}
        "PayBounties" => {}
        "SellOrganicData" => {}

        //Carrier
        "CarrierStats" => {}
        "CarrierJumpRequest" => {}
        "CarrierTradeOrder" => {}
        "CarrierFinance" => {}
        "CarrierJumpCancelled" => {}
        "CarrierDepositFuel" => {}
        "CarrierDockingPermission" => {}
        "CarrierCrewServices" => {}
        "CarrierModulePack" => {}
        "CarrierBankTransfer" => {}

        //Dropship
        "BookDropship" => {}
        "DropshipDeploy" => {}

        //Wing
        "WingInvite" => {}
        "WingJoin" => {}
        "WingAdd" => {}
        "WingLeave" => {}

        //Crew
        "CrewMemberQuits" => {}
        "CrewMemberRoleChange" => {}
        "CrewMemberJoins" => {}
        "EndCrewSession" => {}

        "SellMicroResources" => {}
        "TradeMicroResources" => {}
        "FuelScoop" => {}
        "ReceiveText" => {}
        "Friends" => {}
        "Scanned" => {}
        "LoadGame" => {}
        "SquadronStartup" => {}
        "Music" => {}
        "CodexEntry" => {}
        "Rank" => {}
        "Progress" => {}
        "Reputation" => {}
        "Statistics" => {}
        "Commander" => {}
        "PowerplaySalary" => {}
        "Powerplay" => {}
        "CommitCrime" => {}
        "DockingDenied" => {}
        "HeatWarning" => {}
        "FactionKillBond" => {}
        "MultiSellExplorationData" => {}
        "SwitchSuitLoadout" => {}
        "MaterialTrade" => {}
        "CommunityGoal" => {}
        "ModuleRetrieve" => {}
        "FetchRemoteModule" => {}
        "SendText" => {}
        "SearchAndRescue" => {}
        "HeatDamage" => {}
        "CommunityGoalReward" => {}
        "NavBeaconScan" => {}
        "USSDrop" => {}
        "Interdicted" => {}
        "Promotion" => {}
        "RepairDrone" => {}
        "DataScanned" => {}
        "DatalinkScan" => {}
        "DatalinkVoucher" => {}
        "CockpitBreached" => {}
        "SystemsShutdown" => {}
        "Screenshot" => {}
        "UpgradeWeapon" => {}
        "PowerplayFastTrack" => {}
        "PowerplayCollect" => {}
        "PowerplayDeliver" => {}
        "BookTaxi" => {}
        "SharedBookmarkToSquadron" => {}
        "MaterialDiscovered" => {}
        "SetUserShipName" => {}
        "FCMaterials" => {}
        "CommunityGoalJoin" => {}
        "SupercruiseDestinationDrop" => {}
        "JetConeBoost" => {}
        "AsteroidCracked" => {}
        "EscapeInterdiction" => {}
        "TechnologyBroker" => {}
        "NavBeaconDetail" => {}

        //Jesus
        "Died" => {}
        "Resurrect" => {}
        "SelfDestruct" => {}

        //Redeem
        "ShipyardRedeem" => {}
        "ShipRedeemed" => {}

        //Misc
        "commodities" => {
            let commodities =
                match serde_json::from_value::<crate::edcas::event::commodities::Commodities>(json)
                {
                    Ok(commodities) => commodities,
                    Err(err) => return Err(EdcasError::new(format!("[Commodities]: {}", err))),
                };
            match commodities.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("[Commodities]: {}", err))),
            }
        }
        "ships" => {
            let ships = match serde_json::from_value::<crate::edcas::event::ships::Ships>(json) {
                Ok(ships) => ships,
                Err(err) => return Err(EdcasError::new(format!("Ships: {}", err))),
            };
            match ships.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("Ships: {}", err))),
            }
        }
        "modules" => {
            let modules =
                match serde_json::from_value::<crate::edcas::event::modules::Modules>(json) {
                    Ok(modules) => modules,
                    Err(err) => return Err(EdcasError::new(format!("Modules: {}", err))),
                };
            match modules.insert_into_db(journal_id, client) {
                Ok(_) => return Ok(()),
                Err(err) => return Err(EdcasError::new(format!("Modules: {}", err))),
            }
        }
        "unknown" => {
            error!("UNKNOWN JOURNAL: {}", json);
        }

        "Fileheader" => {
            return Ok(());
        }
        "Shutdown" => {
            return Ok(());
        }
        "" => {
            return Ok(());
        }
        _ => {
            return Err(EdcasError::unknown_event(event.to_string()));
        }
    }
    Err(EdcasError::unimplemented())
}
