# valo-member-bot

todo:<br>
bot.rs: EventHandler実装でのInteraction振り分け時、必要な場合defer()を付ける<br>
bot.rs内でquestion_stateへのデータ挿入を行う<br>
ModalInteractionを受け取ったときにquestion_stateから取得したWebhookDataをpanels::send()に渡し、呼ぶ