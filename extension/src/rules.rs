//! The system prompt: the level and rule sections of the skills, then the answer format.

use crate::types::{Level, Style};
use std::fmt;

/// The description of each conversation style Ollaya chooses from (decisions.rs) and Ollama writes in.
pub fn style_description(s: Style) -> &'static str {
    match s {
        Style::Conciso => "curto e direto: só o necessário, sem rodeio",
        Style::Socratico => "conduz por perguntas e deixa o leitor chegar à conclusão",
        Style::Amigavel => "caloroso e próximo, sem perder a precisão",
    }
}

/// A skill without the section the prompt needs: "<file>: seção <name> não encontrada".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionError {
    pub file: String,
    pub name: String,
}

impl fmt::Display for SectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: seção {} não encontrada", self.file, self.name)
    }
}

impl std::error::Error for SectionError {}

/// The lines from the `heading` line to the next `## ` heading, trimmed.
pub fn section(skill: &str, heading: &str, name: &str, file: &str) -> Result<String, SectionError> {
    let lines: Vec<&str> = skill.split('\n').collect();
    let start = lines
        .iter()
        .position(|l| l.trim_end() == heading)
        .ok_or_else(|| SectionError { file: file.into(), name: name.into() })?;
    let end = lines[start + 1..].iter().position(|l| l.starts_with("## ")).map_or(lines.len(), |i| start + 1 + i);
    Ok(lines[start..end].join("\n").trim().to_string())
}

pub fn build_system_prompt(
    level: Level,
    skill: &str,
    cientifico: &str,
    style: Option<Style>,
) -> Result<String, SectionError> {
    let mut parts = vec![
        "Você revisa texto em português do Brasil segundo as regras abaixo. Aponte só trechos que quebram uma regra."
            .to_string(),
        section(skill, "## Níveis de rigor", "Níveis de rigor", "SKILL.md")?,
        section(skill, "## Regras (em desenvolvimento)", "Regras", "SKILL.md")?,
    ];
    if level == Level::Cientifico {
        parts.push(section(cientifico, "## Regras", "Regras", "cientifico/SKILL.md")?);
    }
    parts.push(format!("Nível pedido: `{level}`. Aplique as regras com o rigor desse nível."));
    if let Some(style) = style {
        parts.push(format!(
            "Estilo da reescrita: `{style}` ({}). Escreva cada replacement nesse estilo.",
            style_description(style)
        ));
    }
    parts.push(
        [
            "Este formato substitui qualquer outro formato de saída citado nas regras acima.",
            "Formato da resposta: responda só com um array JSON, sem texto fora dele. Cada item é um objeto",
            "{\"original\", \"replacement\", \"rule\", \"severity\", \"occurrence\"}:",
            "- original: cópia exata do trecho do texto, sem alterar um caractere;",
            "- replacement: o trecho reescrito;",
            "- rule: o código da regra quebrada (por exemplo PTW-3);",
            "- severity: \"erro\" ou \"aviso\";",
            "- occurrence: qual ocorrência do trecho no texto, contando a partir de 1.",
            "Texto sem problemas: responda []. Não reescreva nada fora dos trechos apontados.",
        ]
        .join("\n"),
    );
    Ok(parts.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{CIENTIFICO, SKILL};

    fn prompt(level: Level, style: Option<Style>) -> String {
        build_system_prompt(level, SKILL, CIENTIFICO, style).unwrap()
    }

    #[test]
    fn inclui_ptw_1_a_13() {
        let p = prompt(Level::Estrito, None);
        for n in 1..=13 {
            assert!(p.contains(&format!("### PTW-{n}")), "PTW-{n}");
        }
    }

    #[test]
    fn exclui_uso_smart_note_e_formato_de_saida() {
        let p = prompt(Level::Estrito, None);
        assert!(!p.contains("## Uso"));
        assert!(!p.contains("## Smart note"));
        assert!(!p.contains("## Formato de saída"));
    }

    #[test]
    fn o_estilo_do_ollaya_entra_no_prompt_so_quando_ha_estilo() {
        assert!(prompt(Level::Leve, Some(Style::Socratico)).contains("Estilo da reescrita: `socrático`"));
        assert!(!prompt(Level::Leve, None).contains("Estilo da reescrita"));
    }

    #[test]
    fn cientifico_embute_cie() {
        assert!(prompt(Level::Cientifico, None).contains("### CIE-1"));
        assert!(!prompt(Level::Leve, None).contains("### CIE-1"));
    }

    #[test]
    fn cientifico_so_embute_as_regras_do_cientifico() {
        let p = prompt(Level::Cientifico, None);
        assert!(p.contains("### CIE-1"));
        for h in ["## Formato de saída", "## Entrada", "## Fluxo"] {
            assert!(!p.contains(h), "{h}");
        }
    }

    #[test]
    fn falha_se_a_secao_regras_do_cientifico_some() {
        let e = build_system_prompt(Level::Cientifico, SKILL, "# x", None).unwrap_err();
        assert_eq!(e.to_string(), "cientifico/SKILL.md: seção Regras não encontrada");
    }

    #[test]
    fn nao_vaza_o_front_matter_do_cientifico() {
        assert!(!prompt(Level::Cientifico, None).contains("name: cientifico"));
    }

    #[test]
    fn declara_o_nivel() {
        assert!(prompt(Level::Leve, None).contains("`leve`"));
    }

    #[test]
    fn falha_se_a_secao_some() {
        let sem_regras = SKILL.replace("## Regras (em desenvolvimento)", "## Outra");
        let e = build_system_prompt(Level::Leve, &sem_regras, CIENTIFICO, None).unwrap_err();
        assert_eq!(e.to_string(), "SKILL.md: seção Regras não encontrada");
        let sem_niveis = SKILL.replace("## Níveis de rigor", "## Outra");
        let e = build_system_prompt(Level::Leve, &sem_niveis, CIENTIFICO, None).unwrap_err();
        assert_eq!(e.to_string(), "SKILL.md: seção Níveis de rigor não encontrada");
    }
}
