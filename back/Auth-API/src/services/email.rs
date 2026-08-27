use once_cell::sync::Lazy;
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::{Resend, Result};
use std::collections::HashMap;

struct EmailTemplate {
    subject: &'static str,
    html: &'static str,
}

static VALIDATION_EMAIL: Lazy<HashMap<&'static str, EmailTemplate>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "en",
        EmailTemplate {
            subject: "Verify your email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verify your email</h1>
                <p style="color:#fff;">Use the code below to confirm your account:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">This code expires in 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "fr",
        EmailTemplate {
            subject: "Vérifie ton email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Vérifie ton email</h1>
                <p style="color:#fff;">Utilise le code ci-dessous pour confirmer ton compte :</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Ce code expire dans 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "es",
        EmailTemplate {
            subject: "Verifica tu correo",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifica tu correo</h1>
                <p style="color:#fff;">Usa el código de abajo para confirmar tu cuenta:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Este código expira en 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "de",
        EmailTemplate {
            subject: "E-Mail bestätigen",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">E-Mail bestätigen</h1>
                <p style="color:#fff;">Verwende den Code unten, um dein Konto zu bestätigen:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Dieser Code läuft in 15 Minuten ab.</p>
            </div>"#,
        },
    );
    m.insert(
        "it",
        EmailTemplate {
            subject: "Verifica la tua email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifica la tua email</h1>
                <p style="color:#fff;">Usa il codice qui sotto per confermare il tuo account:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Questo codice scade tra 15 minuti.</p>
            </div>"#,
        },
    );
    m.insert(
        "pt",
        EmailTemplate {
            subject: "Verifique seu e-mail",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifique seu e-mail</h1>
                <p style="color:#fff;">Use o código abaixo para confirmar sua conta:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Este código expira em 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "ja",
        EmailTemplate {
            subject: "メールアドレスを確認してください",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">メールアドレスを確認してください</h1>
                <p style="color:#fff;">アカウントを確認するには、以下のコードを使用してください：</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">このコードは15分後に失効します。</p>
            </div>"#,
        },
    );
    m.insert(
        "zh",
        EmailTemplate {
            subject: "验证你的邮箱",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">验证你的邮箱</h1>
                <p style="color:#fff;">使用以下代码确认你的账户：</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">此代码将在15分钟后过期。</p>
            </div>"#,
        },
    );
    m.insert(
        "ko",
        EmailTemplate {
            subject: "이메일을 확인해 주세요",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">이메일을 확인해 주세요</h1>
                <p style="color:#fff;">계정을 확인하려면 아래 코드를 사용하세요:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">이 코드는 15분 후에 만료됩니다.</p>
            </div>"#,
        },
    );
    m.insert(
        "ru",
        EmailTemplate {
            subject: "Подтвердите свою почту",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Подтвердите свою почту</h1>
                <p style="color:#fff;">Используйте код ниже, чтобы подтвердить аккаунт:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Этот код действителен 15 минут.</p>
            </div>"#,
        },
    );
    m.insert(
        "uk",
        EmailTemplate {
            subject: "Підтвердьте свою пошту",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Підтвердьте свою пошту</h1>
                <p style="color:#fff;">Використовуйте код нижче, щоб підтвердити обліковий запис:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Цей код дійсний протягом 15 хвилин.</p>
            </div>"#,
        },
    );
    m.insert(
        "pl",
        EmailTemplate {
            subject: "Zweryfikuj swój email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Zweryfikuj swój email</h1>
                <p style="color:#fff;">Użyj poniższego kodu, aby potwierdzić konto:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Ten kod wygasa za 15 minut.</p>
            </div>"#,
        },
    );
    m.insert(
        "tr",
        EmailTemplate {
            subject: "E-postanı doğrula",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">E-postanı doğrula</h1>
                <p style="color:#fff;">Hesabını onaylamak için aşağıdaki kodu kullan:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Bu kod 15 dakika sonra geçersiz olacaktır.</p>
            </div>"#,
        },
    );
    m.insert(
        "nl",
        EmailTemplate {
            subject: "Verifieer je e-mail",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifieer je e-mail</h1>
                <p style="color:#fff;">Gebruik de onderstaande code om je account te bevestigen:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Deze code verloopt over 15 minuten.</p>
            </div>"#,
        },
    );
    m.insert(
        "ar",
        EmailTemplate {
            subject: "تحقق من بريدك الإلكتروني",
            html: r#"<div dir="rtl" style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">تحقق من بريدك الإلكتروني</h1>
                <p style="color:#fff;">استخدم الرمز أدناه لتأكيد حسابك:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;direction:ltr;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">هذا الرمز ينتهي صلاحيته خلال 15 دقيقة.</p>
            </div>"#,
        },
    );
    m.insert(
        "hi",
        EmailTemplate {
            subject: "अपना ईमेल सत्यापित करें",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">अपना ईमेल सत्यापित करें</h1>
                <p style="color:#fff;">अपना खाता पुष्टि करने के लिए नीचे दिए गए कोड का उपयोग करें:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">यह कोड 15 मिनट में समाप्त हो जाएगा।</p>
            </div>"#,
        },
    );
    m.insert(
        "vi",
        EmailTemplate {
            subject: "Xác minh email của bạn",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Xác minh email của bạn</h1>
                <p style="color:#fff;">Sử dụng mã bên dưới để xác nhận tài khoản của bạn:</p>
                <div style="background-color:#080B14;color:#00D9FF;font-size:32px;font-weight:bold;letter-spacing:8px;padding:20px;border-radius:8px;display:inline-block;margin:20px 0;">
                    {{code}}
                </div>
                <p style="color:#888;font-size:12px;">Mã này hết hạn sau 15 phút.</p>
            </div>"#,
        },
    );
    m
});

static VERIFICATION_LINK_EMAIL: Lazy<HashMap<&'static str, EmailTemplate>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "en",
        EmailTemplate {
            subject: "Verify your email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verify your email</h1>
                <p style="color:#fff;">Click the button below to confirm your account:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Verify my email</a>
                <p style="color:#888;font-size:12px;">This link expires in 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "fr",
        EmailTemplate {
            subject: "Vérifie ton email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Vérifie ton email</h1>
                <p style="color:#fff;">Clique sur le bouton ci-dessous pour confirmer ton compte :</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Vérifier mon email</a>
                <p style="color:#888;font-size:12px;">Ce lien expire dans 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "es",
        EmailTemplate {
            subject: "Verifica tu correo",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifica tu correo</h1>
                <p style="color:#fff;">Haz clic en el botón de abajo para confirmar tu cuenta:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Verificar mi correo</a>
                <p style="color:#888;font-size:12px;">Este enlace expira en 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "de",
        EmailTemplate {
            subject: "E-Mail bestätigen",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">E-Mail bestätigen</h1>
                <p style="color:#fff;">Klicke auf den Button unten, um dein Konto zu bestätigen:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">E-Mail bestätigen</a>
                <p style="color:#888;font-size:12px;">Dieser Link läuft in 15 Minuten ab.</p>
            </div>"#,
        },
    );
    m.insert(
        "it",
        EmailTemplate {
            subject: "Verifica la tua email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifica la tua email</h1>
                <p style="color:#fff;">Clicca il pulsante qui sotto per confermare il tuo account:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Verifica la mia email</a>
                <p style="color:#888;font-size:12px;">Questo link scade tra 15 minuti.</p>
            </div>"#,
        },
    );
    m.insert(
        "pt",
        EmailTemplate {
            subject: "Verifique seu e-mail",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifique seu e-mail</h1>
                <p style="color:#fff;">Clique no botão abaixo para confirmar sua conta:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Verificar meu e-mail</a>
                <p style="color:#888;font-size:12px;">Este link expira em 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "ja",
        EmailTemplate {
            subject: "メールアドレスを確認してください",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">メールアドレスを確認してください</h1>
                <p style="color:#fff;">アカウントを確認するには、以下のボタンをクリックしてください：</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">メールアドレスを確認する</a>
                <p style="color:#888;font-size:12px;">このリンクは15分後に失効します。</p>
            </div>"#,
        },
    );
    m.insert(
        "zh",
        EmailTemplate {
            subject: "验证你的邮箱",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">验证你的邮箱</h1>
                <p style="color:#fff;">点击下面的按钮确认你的账户：</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">验证我的邮箱</a>
                <p style="color:#888;font-size:12px;">此链接将在15分钟后过期。</p>
            </div>"#,
        },
    );
    m.insert(
        "ko",
        EmailTemplate {
            subject: "이메일을 확인해 주세요",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">이메일을 확인해 주세요</h1>
                <p style="color:#fff;">계정을 확인하려면 아래 버튼을 클릭하세요:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">이메일 확인</a>
                <p style="color:#888;font-size:12px;">이 링크는 15분 후에 만료됩니다.</p>
            </div>"#,
        },
    );
    m.insert(
        "ru",
        EmailTemplate {
            subject: "Подтвердите свою почту",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Подтвердите свою почту</h1>
                <p style="color:#fff;">Нажмите кнопку ниже, чтобы подтвердить аккаунт:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Подтвердить почту</a>
                <p style="color:#888;font-size:12px;">Эта ссылка действительна 15 минут.</p>
            </div>"#,
        },
    );
    m.insert(
        "uk",
        EmailTemplate {
            subject: "Підтвердьте свою пошту",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Підтвердьте свою пошту</h1>
                <p style="color:#fff;">Натисніть кнопку нижче, щоб підтвердити обліковий запис:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Підтвердити пошту</a>
                <p style="color:#888;font-size:12px;">Це посилання дійсне протягом 15 хвилин.</p>
            </div>"#,
        },
    );
    m.insert(
        "pl",
        EmailTemplate {
            subject: "Zweryfikuj swój email",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Zweryfikuj swój email</h1>
                <p style="color:#fff;">Kliknij poniższy przycisk, aby potwierdzić konto:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Zweryfikuj mój email</a>
                <p style="color:#888;font-size:12px;">Ten link wygasa za 15 minut.</p>
            </div>"#,
        },
    );
    m.insert(
        "tr",
        EmailTemplate {
            subject: "E-postanı doğrula",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">E-postanı doğrula</h1>
                <p style="color:#fff;">Hesabını onaylamak için aşağıdaki butona tıkla:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">E-postamı doğrula</a>
                <p style="color:#888;font-size:12px;">Bu bağlantı 15 dakika sonra geçersiz olacaktır.</p>
            </div>"#,
        },
    );
    m.insert(
        "nl",
        EmailTemplate {
            subject: "Verifieer je e-mail",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Verifieer je e-mail</h1>
                <p style="color:#fff;">Klik op de onderstaande knop om je account te bevestigen:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Mijn e-mail verifiëren</a>
                <p style="color:#888;font-size:12px;">Deze link verloopt over 15 minuten.</p>
            </div>"#,
        },
    );
    m.insert(
        "ar",
        EmailTemplate {
            subject: "تحقق من بريدك الإلكتروني",
            html: r#"<div dir="rtl" style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">تحقق من بريدك الإلكتروني</h1>
                <p style="color:#fff;">انقر على الزر أدناه لتأكيد حسابك:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">تحقق من بريدي</a>
                <p style="color:#888;font-size:12px;">هذا الرابط ينتهي صلاحيته خلال 15 دقيقة.</p>
            </div>"#,
        },
    );
    m.insert(
        "hi",
        EmailTemplate {
            subject: "अपना ईमेल सत्यापित करें",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">अपना ईमेल सत्यापित करें</h1>
                <p style="color:#fff;">अपना खाता पुष्टि करने के लिए नीचे दिए गए बटन पर क्लिक करें:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">मेरा ईमेल सत्यापित करें</a>
                <p style="color:#888;font-size:12px;">यह लिंक 15 मिनट में समाप्त हो जाएगा।</p>
            </div>"#,
        },
    );
    m.insert(
        "vi",
        EmailTemplate {
            subject: "Xác minh email của bạn",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Xác minh email của bạn</h1>
                <p style="color:#fff;">Nhấn vào nút bên dưới để xác nhận tài khoản của bạn:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Xác minh email của tôi</a>
                <p style="color:#888;font-size:12px;">Liên kết này hết hạn sau 15 phút.</p>
            </div>"#,
        },
    );
    m
});

fn email_from() -> String {
    std::env::var("AUTH_EMAIL_FROM").unwrap_or_else(|_| {
        let domain = std::env::var("DOMAIN_EMAIL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "auth.yourdomain.com".to_string());
        format!("ft_transcendence <noreply@{}>", domain)
    })
}

pub async fn send_validation_email(email: String, language: String, code: String) -> Result<()> {
    let api_key = std::env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set");
    let from = email_from();
    let resend = Resend::new(&api_key);
    let to = [email];
    let template = VALIDATION_EMAIL
        .get(language.as_str())
        .unwrap_or_else(|| &VALIDATION_EMAIL["en"]);

    let html = template.html.replace("{{code}}", &code);

    let email = CreateEmailBaseOptions::new(&from, to, template.subject).with_html(&html);
    let _email = resend.emails.send(email).await?;
    Ok(())
}

pub async fn send_verification_link_email(
    email: String,
    language: String,
    url: String,
) -> Result<()> {
    let api_key = std::env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set");
    let from = email_from();
    let resend = Resend::new(&api_key);
    let to = [email];
    let template = VERIFICATION_LINK_EMAIL
        .get(language.as_str())
        .unwrap_or_else(|| &VERIFICATION_LINK_EMAIL["en"]);

    let html = template.html.replace("{{url}}", &url);

    let email = CreateEmailBaseOptions::new(&from, to, template.subject).with_html(&html);
    let _email = resend.emails.send(email).await?;
    Ok(())
}

static PASSWORD_RESET_EMAIL: Lazy<HashMap<&'static str, EmailTemplate>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "en",
        EmailTemplate {
            subject: "Reset your password",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Reset your password</h1>
                <p style="color:#fff;">Click the button below to reset your password:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Reset password</a>
                <p style="color:#888;font-size:12px;">This link expires in 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "fr",
        EmailTemplate {
            subject: "Réinitialise ton mot de passe",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Réinitialise ton mot de passe</h1>
                <p style="color:#fff;">Clique sur le bouton ci-dessous pour réinitialiser ton mot de passe :</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Réinitialiser</a>
                <p style="color:#888;font-size:12px;">Ce lien expire dans 15 minutes.</p>
            </div>"#,
        },
    );
    m.insert(
        "es",
        EmailTemplate {
            subject: "Restablece tu contraseña",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Restablece tu contraseña</h1>
                <p style="color:#fff;">Haz clic en el botón de abajo para restablecer tu contraseña:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Restablecer</a>
                <p style="color:#888;font-size:12px;">Este enlace expira en 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "de",
        EmailTemplate {
            subject: "Passwort zurücksetzen",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Passwort zurücksetzen</h1>
                <p style="color:#fff;">Klicke auf den Button unten, um dein Passwort zurückzusetzen:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Zurücksetzen</a>
                <p style="color:#888;font-size:12px;">Dieser Link läuft in 15 Minuten ab.</p>
            </div>"#,
        },
    );
    m.insert(
        "it",
        EmailTemplate {
            subject: "Reimposta la password",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Reimposta la password</h1>
                <p style="color:#fff;">Clicca il pulsante qui sotto per reimpostare la password:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Reimposta</a>
                <p style="color:#888;font-size:12px;">Questo link scade tra 15 minuti.</p>
            </div>"#,
        },
    );
    m.insert(
        "pt",
        EmailTemplate {
            subject: "Redefina sua senha",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Redefina sua senha</h1>
                <p style="color:#fff;">Clique no botão abaixo para redefinir sua senha:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Redefinir</a>
                <p style="color:#888;font-size:12px;">Este link expira em 15 minutos.</p>
            </div>"#,
        },
    );
    m.insert(
        "ja",
        EmailTemplate {
            subject: "パスワードをリセット",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">パスワードをリセット</h1>
                <p style="color:#fff;">パスワードをリセットするには、以下のボタンをクリックしてください：</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">リセット</a>
                <p style="color:#888;font-size:12px;">このリンクは15分後に失効します。</p>
            </div>"#,
        },
    );
    m.insert(
        "zh",
        EmailTemplate {
            subject: "重置密码",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">重置密码</h1>
                <p style="color:#fff;">点击下面的按钮重置密码：</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">重置</a>
                <p style="color:#888;font-size:12px;">此链接将在15分钟后过期。</p>
            </div>"#,
        },
    );
    m.insert(
        "ko",
        EmailTemplate {
            subject: "비밀번호 재설정",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">비밀번호 재설정</h1>
                <p style="color:#fff;">비밀번호를 재설정하려면 아래 버튼을 클릭하세요:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">재설정</a>
                <p style="color:#888;font-size:12px;">이 링크는 15분 후에 만료됩니다.</p>
            </div>"#,
        },
    );
    m.insert(
        "ru",
        EmailTemplate {
            subject: "Сброс пароля",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Сброс пароля</h1>
                <p style="color:#fff;">Нажмите кнопку ниже, чтобы сбросить пароль:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Сбросить</a>
                <p style="color:#888;font-size:12px;">Эта ссылка действительна 15 минут.</p>
            </div>"#,
        },
    );
    m.insert(
        "uk",
        EmailTemplate {
            subject: "Скидання пароля",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Скидання пароля</h1>
                <p style="color:#fff;">Натисніть кнопку нижче, щоб скинути пароль:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Скинути</a>
                <p style="color:#888;font-size:12px;">Це посилання дійсне протягом 15 хвилин.</p>
            </div>"#,
        },
    );
    m.insert(
        "pl",
        EmailTemplate {
            subject: "Zresetuj hasło",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Zresetuj hasło</h1>
                <p style="color:#fff;">Kliknij poniższy przycisk, aby zresetować hasło:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Resetuj</a>
                <p style="color:#888;font-size:12px;">Ten link wygasa za 15 minut.</p>
            </div>"#,
        },
    );
    m.insert(
        "tr",
        EmailTemplate {
            subject: "Şifreni sıfırla",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Şifreni sıfırla</h1>
                <p style="color:#fff;">Şifreni sıfırlamak için aşağıdaki butona tıkla:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Sıfırla</a>
                <p style="color:#888;font-size:12px;">Bu bağlantı 15 dakika sonra geçersiz olacaktır.</p>
            </div>"#,
        },
    );
    m.insert(
        "nl",
        EmailTemplate {
            subject: "Reset je wachtwoord",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Reset je wachtwoord</h1>
                <p style="color:#fff;">Klik op de onderstaande knop om je wachtwoord te resetten:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Resetten</a>
                <p style="color:#888;font-size:12px;">Deze link verloopt over 15 minuten.</p>
            </div>"#,
        },
    );
    m.insert(
        "ar",
        EmailTemplate {
            subject: "إعادة تعيين كلمة المرور",
            html: r#"<div dir="rtl" style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">إعادة تعيين كلمة المرور</h1>
                <p style="color:#fff;">انقر على الزر أدناه لإعادة تعيين كلمة المرور:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">إعادة التعيين</a>
                <p style="color:#888;font-size:12px;">هذا الرابط ينتهي صلاحيته خلال 15 دقيقة.</p>
            </div>"#,
        },
    );
    m.insert(
        "hi",
        EmailTemplate {
            subject: "पासवर्ड रीसेट करें",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">पासवर्ड रीसेट करें</h1>
                <p style="color:#fff;">पासवर्ड रीसेट करने के लिए नीचे दिए गए बटन पर क्लिक करें:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">रीसेट करें</a>
                <p style="color:#888;font-size:12px;">यह लिंक 15 मिनट में समाप्त हो जाएगा।</p>
            </div>"#,
        },
    );
    m.insert(
        "vi",
        EmailTemplate {
            subject: "Đặt lại mật khẩu",
            html: r#"<div style="background-color:#0F1221;padding:40px;font-family:sans-serif;text-align:center;">
                <h1 style="color:#00D9FF;">Đặt lại mật khẩu</h1>
                <p style="color:#fff;">Nhấn vào nút bên dưới để đặt lại mật khẩu:</p>
                <a href="{{url}}" style="background-color:#00D9FF;color:#0F1221;font-size:18px;font-weight:bold;padding:15px 30px;border-radius:8px;display:inline-block;margin:20px 0;text-decoration:none;">Đặt lại</a>
                <p style="color:#888;font-size:12px;">Liên kết này hết hạn sau 15 phút.</p>
            </div>"#,
        },
    );
    m
});

pub async fn send_password_reset_email(email: String, language: String, url: String) -> Result<()> {
    let api_key = std::env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set");
    let from = email_from();
    let resend = Resend::new(&api_key);
    let to = [email];
    let template = PASSWORD_RESET_EMAIL
        .get(language.as_str())
        .unwrap_or_else(|| &PASSWORD_RESET_EMAIL["en"]);

    let html = template.html.replace("{{url}}", &url);

    let email = CreateEmailBaseOptions::new(&from, to, template.subject).with_html(&html);
    let _email = resend.emails.send(email).await?;
    Ok(())
}
