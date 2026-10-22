// A Lexical rich-text editor, like WhatsApp Web's composer: it keeps the text in its own state and
// redraws the field from it, so a DOM change it does not see is undone.
import { $getRoot, createEditor } from 'lexical';
import { registerRichText } from '@lexical/rich-text';

const editor = createEditor({ namespace: 'e2e', onError: (e) => { throw e; } });
editor.setRootElement(document.getElementById('editor'));
registerRichText(editor);
(window as unknown as { lexicalText: () => string }).lexicalText = () =>
  editor.getEditorState().read(() => $getRoot().getTextContent());
