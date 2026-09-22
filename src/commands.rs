use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "ktünBot komutları:")]
pub enum Command {
    #[command(description = "Botu başlatır.")]
    Start,
    #[command(description = "STEM Bölümünü ve sınıfını seç / profilini gör.")]
    Bolum,
    #[command(description = "Güncel duyuruları listeler. /duyurular [sayı]")]
    Duyurular(String),
    #[command(description = "Ders programını gönderir.")]
    Program,
    #[command(description = "Akademik takvimi gönderir.")]
    Takvim,
    #[command(description = "Bugünün yemekhane menüsü.")]
    Bugun,
    #[command(description = "Dünün yemekhane menüsü.")]
    Dun,
    #[command(description = "Yarının yemekhane menüsü.")]
    Yarin,
    #[command(description = "Bu haftanın yemekhane menüsü.")]
    Hafta,
    #[command(description = "Yemekhane rezervasyon linki.")]
    Rezervasyon,
    #[command(description = "Yemekhane servis saatleri.")]
    Vakit,
    #[command(description = "Yemekhanelerin harita konumları.")]
    Konum,
    #[command(description = "KTÜN Kampüsü güncel hava durumu.")]
    Hava,
    #[command(description = "Son depremler (Kandilli). /deprem veya /deprem konya")]
    Deprem(String),
    #[command(description = "Magnum Opus ders kılavuzu. /magnum [ders]")]
    Magnum(String),
    #[command(description = "Ders notu veya sınav sorusu yükle.")]
    Yukle,
    #[command(description = "Arşivde ders notu veya sınav sorusu ara. /ara <konu veya ders>")]
    Ara(String),
    #[command(description = "Bot hakkında.")]
    Hakkinda,
    #[command(description = "Bu yardım metni.")]
    Yardim,
}

impl Command {
    pub fn parse_count(count_str: String) -> usize {
        let trimmed = count_str.trim();
        if trimmed.is_empty() {
            return 10;
        }
        match trimmed.parse::<usize>() {
            Ok(n) => n.clamp(1, 50),
            Err(_) => 10,
        }
    }
}
