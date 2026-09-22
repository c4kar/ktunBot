use ktunbot::services::earthquake::EarthquakeService;
use ktunbot::services::weather::WeatherService;

#[test]
fn test_wmo_code_mappings() {
    let (desc_clear, emoji_clear) = WeatherService::wmo_code_to_meta(0);
    assert_eq!(desc_clear, "Açık / Güneşli");
    assert_eq!(emoji_clear, "☀️");

    let (desc_rain, emoji_rain) = WeatherService::wmo_code_to_meta(61);
    assert_eq!(desc_rain, "Yağmurlu");
    assert_eq!(emoji_rain, "🌧️");

    let (desc_snow, emoji_snow) = WeatherService::wmo_code_to_meta(71);
    assert_eq!(desc_snow, "Kar Yağışlı");
    assert_eq!(emoji_snow, "❄️");
}

#[test]
fn test_weather_format_html() {
    let report = ktunbot::services::weather::WeatherReport {
        temperature: 21.5,
        apparent_temperature: 20.1,
        humidity: 48,
        wind_speed: 14.2,
        precipitation: 0.0,
        weather_code: 1,
        description: "Çoğunlukla Açık",
        emoji: "🌤️",
    };

    let formatted = WeatherService::format_weather_message(&report);
    assert!(formatted.contains("21.5°C"));
    assert!(formatted.contains("Çoğunlukla Açık"));
    assert!(formatted.contains("<b>KTÜN Yerleşkesi Hava Durumu</b>"));
}

#[test]
fn test_kandilli_raw_parsing() {
    let sample = b"
<pre>
..................TURKIYE VE YAKIN CEVRESINDEKI SON DEPREMLER....................
.....BOLGESEL DEPREM-TSUNAMI IZLEME VE DEGERLENDIRME MERKEZI HIZLI COZUMLERI.....
                                                        Buyukluk
Tarih      Saat      Enlem(N)  Boylam(E) Derinlik(km)  MD   ML   Mw    Yer                                             Cozum Niteligi
---------- --------  --------  -------   ----------    ------------    --------------                                  --------------
2026.09.21 23:49:00  38.9932   27.0640       10.5      -.-  2.0  -.-   TEKKEDERE-BERGAMA (IZMIR)                         Ilksel
2026.09.21 23:41:37  37.6957   32.0278        7.8      -.-  1.5  -.-   SARAYCIK-SEYDISEHIR (KONYA)                       Ilksel
2026.09.21 20:15:10  38.2012   32.5510        5.0      -.-  4.2  -.-   SELCUKLU (KONYA)                                  Ilksel
</pre>";

    let earthquakes = EarthquakeService::parse_kandilli_raw(sample);
    assert_eq!(earthquakes.len(), 3);

    // First earthquake
    assert_eq!(earthquakes[0].time, "23:49:00");
    assert_eq!(earthquakes[0].magnitude, 2.0);
    assert_eq!(earthquakes[0].location, "TEKKEDERE-BERGAMA (IZMIR)");
    assert_eq!(earthquakes[0].severity_emoji(), "🟢");
    assert!(!earthquakes[0].is_konya());

    // Second earthquake (Konya)
    assert_eq!(earthquakes[1].time, "23:41:37");
    assert!(earthquakes[1].is_konya());

    // Third earthquake (Stronger Konya earthquake M 4.2)
    assert_eq!(earthquakes[2].magnitude, 4.2);
    assert_eq!(earthquakes[2].severity_emoji(), "🟡");
    assert!(earthquakes[2].is_konya());
}

#[test]
fn test_earthquake_formatting() {
    let sample = b"
<pre>
Tarih      Saat      Enlem(N)  Boylam(E) Derinlik(km)  MD   ML   Mw    Yer                                             Cozum Niteligi
---------- --------  --------  -------   ----------    ------------    --------------                                  --------------
2026.09.21 20:15:10  38.2012   32.5510        5.0      -.-  4.2  -.-   SELCUKLU (KONYA)                                  Ilksel
</pre>";

    let earthquakes = EarthquakeService::parse_kandilli_raw(sample);
    let msg = EarthquakeService::format_list(&earthquakes, "Konya ve Çevresi Son Depremler");

    assert!(msg.contains("Konya ve Çevresi Son Depremler"));
    assert!(msg.contains("M 4.2"));
    assert!(msg.contains("SELCUKLU (KONYA)"));
    assert!(msg.contains("🟡"));
    assert!(msg.contains("Kandilli Rasathanesi"));
}

#[tokio::test]
async fn test_live_weather_fetch() {
    let service = WeatherService::new();
    let res = service.get_ktun_weather().await;
    assert!(res.is_ok(), "Open-Meteo live API should succeed: {:?}", res.err());
    let report = res.unwrap();
    assert!(report.temperature > -50.0 && report.temperature < 60.0);
    assert!(!report.description.is_empty());
    assert!(!report.emoji.is_empty());
}

#[tokio::test]
async fn test_live_kandilli_fetch() {
    let service = EarthquakeService::new();
    let res = service.get_recent(3).await;
    assert!(res.is_ok(), "Kandilli live feed should succeed: {:?}", res.err());
    let list = res.unwrap();
    assert!(!list.is_empty(), "Should parse at least one recent earthquake");
    assert!(!list[0].location.is_empty());
    assert!(list[0].magnitude > 0.0);
}
