/** Escuta contínua de voz (substitui speech_recognition + recognize_google). */

export class Reconhecedor {
  private rec: any = null;
  private ativo = false;

  iniciar(aoTexto: (texto: string) => void): boolean {
    const Ctor = (window as any).SpeechRecognition ?? (window as any).webkitSpeechRecognition;
    if (!Ctor) {
      console.error("❌ Reconhecimento de voz indisponível neste webview.");
      return false;
    }
    this.ativo = true;
    const rec = new Ctor();
    rec.lang = "pt-BR";
    rec.continuous = true;
    rec.interimResults = false;
    rec.maxAlternatives = 1;

    rec.onresult = (e: any) => {
      for (let i = e.resultIndex; i < e.results.length; i++) {
        if (!e.results[i].isFinal) continue;
        const texto = String(e.results[i][0].transcript ?? "").trim();
        if (texto) {
          console.log(`🗣️ Captado: ${texto}`);
          aoTexto(texto);
        }
      }
    };
    rec.onerror = (e: any) => {
      if (e.error === "not-allowed" || e.error === "service-not-allowed") {
        console.error("Permissão de microfone negada.");
        this.ativo = false;
      } else if (e.error !== "no-speech" && e.error !== "aborted") {
        console.error("Erro de reconhecimento:", e.error);
      }
    };
    rec.onend = () => {
      if (this.ativo) setTimeout(() => this.iniciarSeguro(), 300);
    };
    this.rec = rec;
    this.iniciarSeguro();
    return true;
  }

  private iniciarSeguro(): void {
    try { this.rec?.start(); } catch { /* já iniciado */ }
  }

  parar(): void {
    this.ativo = false;
    try { this.rec?.stop(); } catch { /* ignora */ }
  }
}
