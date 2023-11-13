# Building a Better Foundation for Rocket's Future

<p class="metadata"><strong>
  Posted by <a href="https://sergio.bz">Sergio Benitez</a> on Nov 17, 2023
</strong></p>

Along with the [release of Rocket v0.5], today I'm sharing plans to launch the
Rocket Web Framework Foundation, or [_RWF2_]. The RWF2 is a nonprofit
organization designed to support Rocket and the surrounding ecosystem,
financially and organizationally.

I'm also directly addressing the community's concerns regarding the pace of
Rocket's development, leadership, and release cadence. My hope is to assuage any
and all concerns about Rocket's future. I hope reading this leaves you feeling
confident that Rocket is here to stay, and that the RWF2 is the right step
towards increased community contributions and involvement.

! note: This is a co-announcement [along with release of Rocket v0.5].

[along with release of Rocket v0.5]: ../2023-11-17-version-0.5/
[release of Rocket v0.5]: ../2023-11-17-version-0.5/
[_RWF2_]: https://rwf2.org
[RWF2]: https://rwf2.org

## Background

I released Rocket in 2016 to fanfare. It was lauded as _the_ web framework for
Rust. But in the last few years, I'd be remiss to claim the same. New frameworks
have emerged, Rocket's recent development has been nothing short of erratic, and
four years went by without a major release.

The community rightfully voiced its disappointment and concern. Posts inquired
about the project's status: was it dead? I received copious email ranging from
concern over my well-being, to anger, to requests to transfer the project
entirely. The community ~~wasn't~~ isn't happy with Rocket.

And I get it. I failed to adequately lead the project. I failed to communicate
when it mattered most. I couldn't control the life events that pulled me away
from Rocket and most of my responsibilities, but I could have done more to
communicate what was going on. And I certainly could have done _something_ to
make it easy, make it _possible_ for others to push the project forward in my
absense.

But I did none of that. I couldn't make it happen. And I'm truly, sincerely
sorry.

## A Better Foundation for Rocket's Future

I'd like to make it impossible to repeat these mistakes. That's why today I'm
announcing plans for a new independent nonprofit foundation designed to support
and bolster Rocket's development, increase transparency, and diversify project
leadership: [RWF2].

> The **R**ocket **W**eb **F**ramework **F**oundation, _RWF2_, is a
> <abbr title="RWF2 is granted 501(c)(3) status via its fiscal host, the OCF.">
> 501(c)(3) nonprofit</abbr> and <a href="https://opencollective.com/rwf2">
> collective</a> that supports the development and community of free and open
> source software, like <a href="https://rocket.rs">Rocket</a>, as well as
> education for a more secure web.

Moving forward, the RWF2 will be responsible for governing Rocket and dictating
its trajectory. The goal is to distribute control of the project and prohibit
one person from being able to stall its development. The RWF2 will also act as a
vehicle for tax-deductible contributions, funds management, and development
grant distribution, all with the aim of increasing high-quality contributions
and educational material.

In summary, the RWF2 exists to enable:

  * **Diversified Leadership**

    Key responsibilities, such as releases, security, infrastructure, and
    community engagement will be distributed to community members under the
    umbrella of the foundation.

  * **Tax-Deductible Contributions**

    Because the RWF2 is a 501(c)(3) organization, contributions are
    tax-deductible. We particularly hope this encourages corporate sponsorship,
    especially from those who depend on Rocket. As a nonprofit, the RWF2 must
    transparently manage and disburse all funds.

  * **Development Grants**

    A key use for contributions is the foundation's sponsorship and
    administration of µGrants: small (≤ $1k) grants for concrete work on
    Rocket or related projects. Compensation is staged upon completion of
    predefined milestones and quality requirements.

  * **Increased Transparency**

    Milestones, release schedules, and periodic updates form part of the
    foundation's responsibilities. The aim is to keep the community informed on
    Rocket's development and plans, making it easier to get and remain involved.

  * **Educational Resource Expansion**

    The RWF2 aims to enhance the accessibility of educational resources,
    training, and mentorship for web application security, especially for
    traditionally marginalized groups. Our focus lies in delivering
    high-quality, practical materials for building secure web applications.

## What's Happening Now

There's a lot to do to realize these goals, but the process starts today. Here's
what's being done now:

  0. **Open Sponsorship**

     Starting now, you can sponsor the RWF2 through [GitHub Sponsors] or
     [Open Collective]. Tiers are still a work in progress, but for now, consider
     all tiers on Open Collective, and Bronze+ tiers on GitHub, as intended for
     corporate sponsors. Note that only contributions made directly via Open
     Collective are guaranteed to be tax-deductible.

     A special shout out to `@martynp`, `@nathanielford`, and `@wezm` for
     jumping the gun in the best of ways and sponsoring the RWF2 via GitHub
     ahead of schedule. Thank you!

  0. **Team Assembly**

     Initially, RWF2 governance will be exceedingly simple and consist of a
     president <small>(hi!)</small> and a handful of team leads. Individuals can
     fill multiple positions, though the intent is for every position to be held
     by a different individual. Positions are by appointment, either by the
     presiding team lead, by the president in their absence, and by other team
     leads in the president's absence.

     The initial teams and their responsibilities are listed below. If you're
     interested in leading any of the teams (or another team you think should
     exist), please reach out via the [Matrix channel] or directly via
     [foundation@rwf2.org](mailto:foundation@rwf2.org).

     - *Maintenance*

       Reviews issues, pull requests, and discussions, and acts on them as
       necessary. This largely means triaging issues, closing resolved or
       duplicate issues and discussions, closing or merging stale or approved
       pull requests, respectively, and pinging the appropriate individuals to
       prevent issues or PRs from becoming stale.

     - *Release*

       Publishes code and documentation releases. This includes partitioning
       commits according to scope and impact on breakage, writing and updating
       CHANGELOGs, and testing and publishing new releases and their
       documentation.

     - *Knowledge*

       Creates, maintains and improves materials that help others learn about
       Rocket or web security. This includes documentation like API docs and the
       Rocket guide, code such as examples and tutorials, and materials for live
       or in-person education.

     - *Community*

       Keeps the community adjourned on happenings. This involves writing
       periodic project updates as well as digesting and communicating
       development milestones and schedules to a broad audience.

     - *Infrastructure*

       Maintains infrastructure including: building, testing, and release
       scripts, static site generation, CI and other automated processes, and
       domain registrar and cloud computing services.

  0. **Transfer of Assets**

     The majority of Rocket's assets, including its domain, website, source
     code, and associated infrastructure, are managed under personal accounts.
     All assets are being transferred to foundation-owned accounts, and access
     will be given to the appropriate teams. The [migration project on GitHub]
     is tracking the progress of asset migration.

  0. **Process Documentation**

     Some of Rocket's core processes, including releases and site building, are
     generally inaccessible to others. These will be documented, and access will
     be granted to the appropriate teams.

  0. **Open Planning & Development**

     While Rocket's development has generally been done in the open through
     GitHub issues, PRs, and projects, little has been done to publicize those
     efforts. Furthermore, _planning_ has largely been a closed process. Moving
     forward, planning will be done in the open, and the community team will be
     engaged to publicize development efforts and progress.

[GitHub Sponsors]: https://github.com/sponsors/rwf2
[Open Collective]: https://opencollective.com/rwf2
[Matrix channel]: https://chat.mozilla.org/#/room/#rocket:mozilla.org
[migration project on GitHub]: https://github.com/orgs/rwf2/projects/1

## What's Coming Soon

  * **µGrants**

    The µGrant specification is a work-in-progress. We simulatenously want to
    encourage and financially incentivize high-quality contributions while not
    disincentivizing existing contributors. This is a delicate balance, and we
    want to take the time to get it right. To get involved, see the current
    [draft proposal](https://github.com/rwf2/rwf2.org/blob/master/docs/micro-grants.md) and
    share your thoughts in the [GitHub discussion](https://github.com/orgs/rwf2/discussions/8).
    As soon as we have a specification that feels fair, the first µGrants will
    be offered.

  * **Foundation Website**

    The [RWF2 website](https://rwf2.org) as it stands is a placeholder for a
    more fully featured website. Besides articulating the foundation's mission
    and goals, the RWF2's website will also serve as a source of truth for the
    status of and means to engaging with ongoing projects, grants, and finances.

  * **Membership**

    While certainly premature at this point in time, a consideration for the
    future comes in the form of foundation _membership_ whereby governance is
    expanded to include foundation _members_. The [governance proposal
    document](https://github.com/rwf2/rwf2.org/blob/master/docs/governance.md)
    has one take on how this might work. Until such a proposal is accepted,
    governance will follow the president + teams model articulated above.

## How to Get Involved

The RWF2 represents a conscious effort to transfer control of Rocket from an
individual (me) to the community (you). Without your involvement, the RWF2
ceases to exist. If you're excited about Rocket or the foundation, or simply
want to see Rocket continue to exist and flourish, please get involved.

  * **Join the Discussion**

    Communicate with us via the [Matrix channel], via [GitHub
    discussions](https://github.com/orgs/rwf2/discussions), or via email at
    [foundation@rwf2.org](mailto:foundation@rwf2.org). The foundation bring-up
    itself is designed to be collaborative, and any input you have is
    invaluable.

  * **Make a Contribution**

    Any easy way to get involved is to financially contribute. You can sponsor
    the RWF2 through [GitHub Sponsors] or [Open Collective]. If your company
    uses Rocket, encourage it to sponsor the project through the foundation.

  * **Become a Team Lead**

    If you're interested in leading or learning more about any one of the
    *Maintenance*, *Release*, *Knowledge*, *Community*, or *Infrastructure*
    teams, or think another team should exist, please get in touch via the
    [Matrix channel] or via email at [foundation@rwf2.org](mailto:foundation@rwf2.org).

I'm excited for this next step in Rocket's history, and I hope you'll join me in
making it a success.
