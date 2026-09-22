use ktunbot::services::cafeteria_scraper::parse_menu_html;

#[test]
fn test_parse_menu_html_sample() {
    let sample_html = r#"
    <table class="menu-calendar">
        <tr>
            <th><center>Pazartesi</center></th>
            <th><center>Salı</center></th>
            <th><center>Çarşamba</center></th>
            <th><center>Perşembe</center></th>
            <th><center>Cuma</center></th>
        </tr>
        <tr>
            <td class=""></td>
            <td class="">
                <h4>1 Eyl&#252;l</h4>
                <div class="container">
                    <div class="row">
                        <div class="col-12">
                            <ul>
                                <li>TEL ŞEHRİYE &#199;ORBASI</li>
                                <li>SEBZELİ KAŞARLI K&#214;FTE</li>
                                <li>PİRİN&#199; PİLAVI</li>
                                <li>AYRAN</li>
                            </ul>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-12">
                            <div class="calorie-info">
                                <b>Toplam Kalori: <span class="calorie-value">590</span></b>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
            <td class=""></td>
            <td class=""></td>
            <td class=""></td>
        </tr>
    </table>
    "#;

    let data = parse_menu_html(sample_html, 2026, 9, "2026-09-01T00:00:00+03:00");
    assert_eq!(data.year, 2026);
    assert_eq!(data.month, "Eylül");
    assert_eq!(data.menu.len(), 1);

    let item = &data.menu[0];
    assert_eq!(item.date, "01 Eylül");
    assert_eq!(item.day_of_week, "Salı");
    assert_eq!(item.foods.len(), 4);
    assert_eq!(item.foods[0], "TEL ŞEHRİYE ÇORBASI");
    assert_eq!(item.foods[1], "SEBZELİ KAŞARLI KÖFTE");
    assert_eq!(item.foods[2], "PİRİNÇ PİLAVI");
    assert_eq!(item.foods[3], "AYRAN");
    assert_eq!(item.totalcalorie, "590 Kcal");
    assert!(!item.closed);
}
